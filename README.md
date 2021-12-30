# reverse_internet
功能：让内网服务器通过连接该机器的电脑上网  
使用场景：安装、更新软件  
已知问题：偶发响应为空  

名词解释：  
inside：内网服务器  
outside：通过vpn等连接内网服务器的机器  

默认参数运行：  
inside：./reverse_internet  
outside：reverse_internet.exe --mode o --inside_addr 内网机器ip:50001  

查看帮助：./reverse_internet --help  
