## 使い方
### フィヨルドブートキャンプアプリのJWTを取得
```shell
% curl -XPOST -D - https://bootcamp.fjord.jp/api/session -d login_name=your_email -d password=your_password
```

### 取得したJWTを環境変数に指定して起動
```shell
% FJORD_JWT_TOKEN=your_jwt cargo run
```

### うまく動かないとき
JWTの有効期限が切れている可能性があるので再取得してみてください。
