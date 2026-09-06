//! 二进制入口：调用`run()`输出全部词典文件

/// 程序入口
fn main() -> Result<(), Box<dyn std::error::Error>> {
	tiger_code_sha1::run()
}
