# tradu

A simple tool to translate using "DeepL"

## Configuration

```bash
mkdir -p ~/.config/tradu/
cp tradu.toml ~/.config/tradu/
```

The configuration file `tradu.toml`,

```bash
log_level: error
base_url: api-free.deepl.com
endpoint: v2/translate
auth_key: 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'
source_lang: ES
target_lang: EN
split_sentences: 1
preserve_formatting: false
formality: default
```

Add the `Auth Key` from DeepL to begin to translate.

## use

Simple run `tradu` with a text to translate.

