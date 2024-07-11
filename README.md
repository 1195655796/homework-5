# Geektime Rust 语言训练营

第五周：连锁闪电：构建高性能互联网应用

作业5:
0. 注意事项
 如果遇到用户登录expired要修改index.html里的token，
 chat_server内models内的main函数运行时，要修改models内的config文件，File::open(r#"D:\homework5\chat_server\chat.yml"#)中的链接要修改为chat_server内的chat.yml文件的绝对路径，
 notify_server运行时则要修改src中的config文件，r#"D:\homework5\notify_server\notify.yml"#要修改为notify_server中notify.yml的绝对路径
1. utoipa 剩下的代码支持：
支持
signup_handler,
signin_handler,
list_chat_handler,
create_chat_handler,
get_chat_handler,
list_message_handler,
file_handler,
send_message_handler,
upload_handler,
update_chat_handler,
delete_chat_handler
在swagger_UI访问；
2. notify-server bug:
如果用户退出，会提示 
User 1 connection dropped and cleaned up
 INFO notify_server::sse: Confirmed: User 1 removed from DashMap, bug解决，

3. chat service里的API：
update_chat_handler,
delete_chat_handler已完成

4. 拓展notify service：
chat name update 示例在test.rest文件中，会通知chat updated

5.本周涉及的第三方crate 阅读文档和examples：
已读#