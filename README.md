# Tiger Code Sha1

基于[虎码](https://www.tiger-code.com)魔改的Rime输入方案，小狼毫、鼠须管、Fcitx5通用

## 特性

- **中英混输**：没有自动上屏，支持中英文混输
- **拼音反查**：通过`|`前缀触发拼音反查功能
- **简化字表**：默认仅收录国标简体字，每个字仅保留一个编码
- **中文词组**：基于词频自动补充常用词组，并支持手动加词
- **英文词典**：基于词频和标准拼写库自动生成英文单词候选
- **更多符号**：全角模式自带常用特殊符号和emoji（按F8切换）

## 部分文件结构

`()`内为运行时生成

```text
./
├ tiger_code_sha1.schema.yaml		# 主输入方案
├ (tiger_code_sha1.dict.yaml)		# 中文词典
├ pinyin_lookup.schema.yaml			# 拼音反查伪方案
├ (pinyin_lookup.dict.yaml)			# 拼音反查词典
├ symbols.yaml						# 符号表
├ default.custom.yaml				# Rime默认配置
├ weasel.custom.yaml				# 小狼毫界面定制（其他框架可忽略）
├ lua/commit_raw_symbol.lua			# 符号输入体验优化
└ dicts/							# 词典生成代码和基础数据
	└ README.md						# 开发文档
```

## 使用方法

1. 安装依赖
2. 清空Rime用户文件夹
3. 执行
```ps1
git clone --depth=1 https://github.com/Sha1rholder/tiger-code-sha1.git "$env:APPDATA\Rime" # 请勿必加上`--depth=1`，本repo的git history很爆炸
cd "$env:APPDATA\Rime\dicts"
uv run data\get_wordfreq.py
cargo run --release
```
4. 在Weasel控制面板中选择`tiger_code_sha1`

fcitx5目录为`~/.local/share/fcitx5/rime`，鼠须管目录为`~/Library/Rime`

## 介绍网站

[在线阅读](https://sha1rholder.github.io/tiger-code-sha1/)，内容来自[site/intro.md](site/intro.md)，使用Pandoc转换为单页HTML

先按[Pandoc安装说明](https://pandoc.org/installing.html)安装Pandoc，Ubuntu/Debian可运行`sudo apt-get install pandoc`，Fedora可运行`sudo dnf install pandoc`

在仓库根目录构建并预览（预览需要Python 3）：

```sh
bash site/build.sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory site/dist
```

浏览器打开`http://127.0.0.1:8000/`即可预览，生成的`site/dist/`不提交到仓库

GitHub仓库的Settings → Pages → Build and deployment → Source选择GitHub Actions。`main`分支的`site/`或Pages工作流修改会自动构建部署，也可手动运行工作流；PR只构建验证

## 致谢

- [虎码输入法](https://www.tiger-code.com)
- [Rime](https://rime.im)
- [通用规范汉字表](https://github.com/shengdoushi/common-standard-chinese-characters-table)
- [English Speller Database](https://wordlist.aspell.net)
- [wordfreq](https://github.com/rspeer/wordfreq)
