use std::net::SocketAddr;
use std::process::exit;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

///外网运行
#[derive(Debug)]
pub struct OutsideParams {
    ///inside连接地址：ip:port
    inside_addr: String,
    ///socks上网端口
    socks_port: u16,
}

impl OutsideParams {
    pub fn new(inside_addr: String, socks_port: u16) -> OutsideParams {
        OutsideParams {
            inside_addr,
            socks_port,
        }
    }

    pub fn copy(&self) -> OutsideParams {
        OutsideParams {
            inside_addr: self.inside_addr.clone(),
            socks_port: self.socks_port.clone(),
        }
    }
}

///运行函数
pub async fn run(params: OutsideParams) {
    let params_copy = params.copy();
    let server_result = TcpStream::connect(params_copy.inside_addr.clone()).await;
    let (mut sr, mut sw) = server_result.expect("与inside建立连接失败").into_split();

    tokio::spawn(async move {
        // 心跳，避免有些网络环境通信中断
        loop {
            sw.write_i8(1).await.expect("与inside端通信失败");
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    tokio::spawn(async move {
        // 与inside建立连接进行管理
        loop {
            match sr.read_i32().await {
                Ok(count) => {
                    dowith_inside(params_copy.copy()).await;
                    println!("链接计数：{}", count);
                }
                Err(_) => {
                    println!("与inside端连接中断");
                    exit(1);
                }
            };
        }
    });

    // 创建socks代理流量出口
    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), params.socks_port);
    // 未使用认证
    socks5_proxy::server::new(addr, None)
        .expect("创建socks5代理失败，请换个端口")
        .run()
        .await
        .unwrap();
}

async fn dowith_inside(params: OutsideParams) {
    // 与inside建立连接
    let server_result = TcpStream::connect(params.inside_addr.clone()).await;
    let (mut ir, mut iw) = server_result.expect("与inside建立连接失败").into_split();

    // 与下级socks代理建立连接
    let server_result = TcpStream::connect(format!("127.0.0.1:{}", params.socks_port)).await;
    let (mut sr, mut sw) = server_result.expect("与下级socks建立连接失败").into_split();

    tokio::spawn(async move {
        tokio::io::copy(&mut ir, &mut sw).await.unwrap_or(0);
    });

    tokio::spawn(async move {
        tokio::io::copy(&mut sr, &mut iw).await.unwrap_or(0);
    });
}
