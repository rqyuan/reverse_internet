use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};

///没实现认证
pub struct HttpArgs {
    ///监听端口
    pub port: u16,
    ///下一个http代理的ip:port地址
    pub next_http: String,
}

///http代理
pub async fn run(http_args: HttpArgs) {
    let port = http_args.port;
    let next_http = http_args.next_http;

    if next_http.is_empty() && port == 0 {
        return;
    }
    if port == 0 {
        return;
    }

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (client, _) = listener.accept().await.unwrap();
        let http_upper_clone = next_http.clone();
        tokio::spawn(async move {
            handle_connection(client, http_upper_clone).await;
        });
    }
}

// 处理链接、转发
async fn handle_connection(client: TcpStream, next_http: String) {
    let mut read_buf = [0; 4096];

    //分离读写
    let (mut cr, mut cw) = client.into_split();
    let request_buf_sieze = cr.read(&mut read_buf).await.unwrap();

    // 处理多余的换行,避免写入sw没有结束符
    let mut idx = request_buf_sieze;
    while idx > 0 {
        if read_buf[idx - 2] == 13u8 && read_buf[idx - 1] == 10u8 {
            idx = idx - 2;
        } else {
            break;
        }
    }

    //读取请求数据
    let request = String::from_utf8_lossy(&read_buf[..idx]);
    //读取完数据后就断开链接,避免http请求不会结束
    let request = request.replace("Proxy-Connection: keep-alive", "Connection: close");
    let request = request.replace("Connection: keep-alive", "Connection: close");
    // println!("{}", request);

    //保存链接类型
    let first_line = request.lines().next().unwrap_or("").to_string();
    if first_line == "" {
        return;
    }

    let first_arr: Vec<&str> = first_line.split(" ").collect();
    let request_type = first_arr[0];

    //保存目标服务器的 域名:端口
    let mut target_host = String::new();

    if !next_http.is_empty() {
        // 多级代理
        target_host = next_http.clone();
    } else {
        for line in request.lines() {
            let data: Vec<&str> = line.split(": ").collect();

            if data[0].eq_ignore_ascii_case("host") {
                target_host = data[1].to_string();
                if !target_host.contains(":") {
                    target_host += ":80";
                }
                break;
            }
        }
    }

    if target_host.is_empty() {
        return;
    };

    // 建立连接
    let server_result = TcpStream::connect(target_host).await;
    let server: TcpStream = match server_result {
        Ok(x) => x,
        Err(_) => {
            // println!("建立连接失败，href：{}", request);
            return;
        }
    };
    //分离读写
    let (sr, mut sw) = server.into_split();

    if !next_http.is_empty() {
        //多级代理
        sw.write(request.as_bytes()).await.unwrap_or(0);
        sw.write(b"\r\n\r\n").await.unwrap_or(0);

        tokio::spawn(async move {
            copy(cr, sw).await;
        });

        tokio::spawn(async move {
            copy(sr, cw).await;
        });
    } else {
        if request_type == "CONNECT" {
            // 连接目标成功之后，返回下面内容，表示 通知浏览器连接成功
            cw.write(b"HTTP/1.0 200 Connection Established\r\n\r\n")
                .await
                .unwrap_or(0);

            // 客户端 -> 服务器
            tokio::spawn(async move {
                copy(cr, sw).await;
            });

            // 服务器 -> 客户端
            tokio::spawn(async move {
                copy(sr, cw).await;
            });
        } else {
            //http请求
            sw.write(request.as_bytes()).await.unwrap_or(0);
            sw.write(b"\r\n\r\n").await.unwrap_or(0);

            copy(sr, cw).await;
            sw.shutdown().await.unwrap_or(());
        }
    }
}

async fn copy(mut read: OwnedReadHalf, mut write: OwnedWriteHalf) {
    // Connection: keep-alive 类型的链接,copy会一直等待
    tokio::io::copy(&mut read, &mut write).await.unwrap_or(0);
    write.shutdown().await.unwrap_or(());
}
