# am-rate-bot

Armenia currency exchange rates telegram bot.

<img width="470" alt="am-rate-bot" src="https://github.com/user-attachments/assets/67903e8c-d089-4ff5-bc46-3405d59f0b08">


## Setup

```shell
make fix-config
```

You have to set `TELOXIDE_TOKEN` env to your telegram bot token in `config/bot.env`.

## Run

```shell
docker compose -f ./compose.polling.yaml up -d
```

## License

[![GNU GPLv3 Image](https://www.gnu.org/graphics/gplv3-127x51.png)](https://www.gnu.org/licenses/gpl-3.0.en.html)
