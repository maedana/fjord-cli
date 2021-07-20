## 使い方
### Rustインストール
https://www.rust-lang.org/ja/tools/install

### フィヨルドブートキャンプアプリのJWTを取得
```shell
% curl -XPOST -D - https://bootcamp.fjord.jp/api/session -d login_name=your_email -d password=your_password
```

### 取得したJWTを環境変数に指定して起動
```shell
% FJORD_JWT_TOKEN=your_jwt cargo run
```

### 操作方法
- h,lでタブ切り替え
- j,kで上下に移動
- oでブラウザで対象を開く
- qで終了

## トラブルシューティング
### うまく動かないとき
JWTの有効期限が切れている可能性があるので再取得してみてください。
