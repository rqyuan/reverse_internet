use clap::{App, Arg};

use reverse_internet::{inside, outside};

/// 环境说明：外网机器（vpn、代理等）能连接内网机器
/// 功能说明：内外网机器通过该应用建立连接，使内网机器能通过外网机器上网
/// 已知问题：偶发响应为空
#[tokio::main]
async fn main() {
    let matches = App::new("args")
        .arg(
            Arg::with_name("mode")
                .required(true)
                .takes_value(true)
                .long("mode")
                .default_value("i")
                .help("运行模式：i：在内网运行，o：在外网运行")
                .display_order(1),
        )
        .arg(
            Arg::with_name("socks_port")
                .required(false)
                .takes_value(true)
                .long("socks_port")
                .default_value("50000")
                .help("socks监听端口")
                .display_order(2),
        )
        .arg(
            Arg::with_name("port")
                .required(false)
                .takes_value(true)
                .long("port")
                .default_value("50001")
                .help("inside和outside通信端口")
                .display_order(3),
        )
        .arg(
            Arg::with_name("inside_addr")
                .required(false)
                .takes_value(true)
                .long("inside_addr")
                .default_value("127.0.0.1:50001")
                .help("inside连接地址")
                .display_order(4),
        )
        .arg(
            Arg::with_name("socks_port_out")
                .required(false)
                .takes_value(true)
                .long("socks_port_out")
                .default_value("50002")
                .help("socks上网端口")
                .display_order(5),
        )
        .get_matches();

    let mode = matches.value_of("mode").expect("获取mode失败").to_string();
    let socks_port: u16 = matches
        .value_of("socks_port")
        .expect("获取socks监听端口失败")
        .parse()
        .unwrap();
    let port: u16 = matches
        .value_of("port")
        .expect("获取inside和outside通信端口失败")
        .parse()
        .unwrap();
    let inside_params = inside::InsideParams::new(port, socks_port);

    let inside_addr = matches
        .value_of("inside_addr")
        .expect("获取inside连接地址失败")
        .to_string();
    let socks_port_out: u16 = matches
        .value_of("socks_port_out")
        .expect("获取socks上网端口失败")
        .parse()
        .unwrap();

    let outside_params = outside::OutsideParams::new(inside_addr, socks_port_out);

    if mode == "i" {
        println!(
            "设置代理：export all_proxy=socks5://127.0.0.1:{}",
            socks_port
        );
        println!("取消代理：unset all_proxy");
        println!(
            "outside运行命令:reverse_internet.exe --mode o --inside_addr 内网机器ip:{}",
            port
        );
        inside::run(inside_params).await;
    } else {
        outside::run(outside_params).await;
    }
}
