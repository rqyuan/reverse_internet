use clap::{App, Arg};

use reverse_internet::{inside, outside};

/// 环境说明：外网机器（vpn、代理等）能连接内网机器
/// 功能说明：内外网机器通过该应用建立连接，使内网机器能通过外网机器上网
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
            Arg::with_name("proxy_port")
                .required(false)
                .takes_value(true)
                .long("proxy_port")
                .default_value("50000")
                .help("i：http代理监听端口")
                .display_order(2),
        )
        .arg(
            Arg::with_name("inside_outside_port")
                .required(false)
                .takes_value(true)
                .long("inside_outside_port")
                .default_value("50001")
                .help("i：inside和outside通信端口")
                .display_order(3),
        )
        .arg(
            Arg::with_name("inside_addr")
                .required(false)
                .takes_value(true)
                .long("inside_addr")
                .default_value("127.0.0.1:50001")
                .help("o：inside连接地址")
                .display_order(4),
        )
        .arg(
            Arg::with_name("proxy_out_port")
                .required(false)
                .takes_value(true)
                .long("proxy_out_port")
                .default_value("60000")
                .help("o：程序使用端口")
                .display_order(5),
        )
        .arg(
            Arg::with_name("proxy_out_addr")
                .required(false)
                .takes_value(true)
                .long("proxy_out_addr")
                .default_value("")
                .help("o：外部http代理地址，例如：localhost:60003。有值则proxy_out_port不生效")
                .display_order(6),
        )
        .get_matches();

    let mode = matches.value_of("mode").expect("获取mode失败").to_string();
    let proxy_port: u16 = matches
        .value_of("proxy_port")
        .expect("获取http监听端口失败")
        .parse()
        .unwrap();
    let inside_outside_port: u16 = matches
        .value_of("inside_outside_port")
        .expect("获取inside和outside通信端口失败")
        .parse()
        .unwrap();
    let inside_params = inside::InsideParams::new(inside_outside_port, proxy_port);

    let inside_addr = matches
        .value_of("inside_addr")
        .expect("获取inside连接地址失败")
        .to_string();
    let proxy_out_port: u16 = matches
        .value_of("proxy_out_port")
        .expect("获取程序使用端口失败")
        .parse()
        .unwrap();
    let proxy_out_addr = matches
        .value_of("proxy_out_addr")
        .expect("获取外部http代理地址失败")
        .to_string();

    let outside_params = outside::OutsideParams::new(inside_addr, proxy_out_port, proxy_out_addr);

    if mode == "i" {
        println!(
            "设置代理：export http_proxy=http://127.0.0.1:{} https_proxy=http://127.0.0.1:{}",
            proxy_port, proxy_port
        );
        println!("取消代理：unset http_proxy https_proxy");
        println!(
            "outside运行命令:reverse_internet.exe --mode o --inside_addr 内网机器ip:{}",
            inside_outside_port
        );
        inside::run(inside_params).await;
    } else {
        outside::run(outside_params).await;
    }
}
