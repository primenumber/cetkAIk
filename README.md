cetkAIk
====

机戦のAI

# 使い方
現状、`CetkaikEngine` トレイトを実装している AI 同士を戦わせることができる。

## デバッグメッセージを最小限にして 10 試合走らせるためのコマンド

```
cargo run --release -- --hide-move --hide-board --hide-ciurl -c 10
```

または

```
cargo run --release -- --quiet -c 10
```

## Tun2Kik1 と Greedy を戦わせるためのコマンド
```
cargo run --release -- --ia-side tunkik --a-side greedy --count 100
```