use std::process::exit;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

///内网运行
#[derive(Debug, Copy, Clone)]
pub struct InsideParams {
    ///代理流量进入端口
    proxy_port: u16,
    ///inside、outside通信端口
    inside_outside_port: u16,
}

impl InsideParams {
    pub fn new(inside_outside_port: u16, proxy_port: u16) -> InsideParams {
        InsideParams {
            inside_outside_port,
            proxy_port,
        }
    }
}

///运行函数
pub async fn run(params: InsideParams) {
    // inside和outside通信：当inside接收到流量后，通知outside新增连接进行请求转发
    let (inside_outside_tx, inside_outside_rx) = mpsc::channel::<i8>(100);
    // 接收请求
    let (client_tx, client_rx) = mpsc::channel::<TcpStream>(100);
    let w1 = run_port(params.clone(), client_tx, inside_outside_rx);
    let w2 = run_socks(params.clone(), client_rx, inside_outside_tx);
    tokio::join!(w1, w2);
}

/// 运行inside和outside通信
async fn run_port(params: InsideParams, tx: Sender<TcpStream>, mut port_rx: Receiver<i8>) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", params.inside_outside_port))
        .await
        .unwrap();

    // 第一条outside发送的请求用来进行管理
    let (client, _) = listener.accept().await.unwrap();
    client.set_nodelay(true).expect("设置管理连接失败");
    let (mut cr, mut cw) = client.into_split();

    tokio::spawn(async move {
        // 心跳，避免有些网络环境通信中断
        loop {
            match cr.read_i8().await {
                Ok(_) => {}
                Err(_) => {
                    println!("与outside端通信失败");
                    exit(1);
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut count: i32 = 1;
        while let Some(_i) = port_rx.recv().await {
            // 通知outside建立新的链接
            cw.write_i32(count).await.expect("通知outside失败");
            count = count + 1;
        }
    });

    // 把outside连接放到队列中备用
    loop {
        let (client, _) = listener.accept().await.unwrap();
        tx.send(client).await.expect("写入outside队列失败");
    }
}

///运行端口监听，进行tcp流量转发
async fn run_socks(params: InsideParams, mut rx: Receiver<TcpStream>, port_tx: Sender<i8>) {
    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", params.proxy_port))
        .await
        .expect("运行http监听失败");

    loop {
        let (socks_client, _) = tcp_listener.accept().await.unwrap();
        //读取进来的流量
        let (mut ir, mut iw) = socks_client.into_split();

        // 通知outside发送新的连接，进行流量转发
        port_tx.send(1).await.expect("写入inside失败");

        //进行内外网流量转发
        let (mut pr, mut pw) = rx
            .recv()
            .await
            .expect("获取outside端连接失败，请重启双端应用")
            .into_split();

        // println!("开始处理转发");
        tokio::spawn(async move {
            tokio::io::copy(&mut ir, &mut pw).await.unwrap_or(0);
        });

        tokio::spawn(async move {
            tokio::io::copy(&mut pr, &mut iw).await.unwrap_or(0);
        });
    }
}
