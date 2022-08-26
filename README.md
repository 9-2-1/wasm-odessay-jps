在线演示

<https://six-6.gitee.io/wasm-odessay-jps/example.html>

准备工作

``` batchfile
rem 必要的
cargo install wasm-pack
rem 为了减小体积，用了nightly的一些功能
rustup default nightly
rem 如果要浏览器演示，建议
npm install -g five-server
rem 或者
yarn global add five-server
```

用nodejs演示
``` batchfile
rem 带调试输出
call debug.bat
rem 或者，不带调试输出
call build.bat
rem 测试数据在test.js里，只有一组...
node test.js
```

用浏览器演示
``` batchfile
rem 带调试输出
call wdebug.bat
rem 或者，不带调试输出
call wbuild.bat
rem 拉起本地服务器
five-server
rem 接着，打开example.html
```
