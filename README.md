# am-rate-bot

Armenia currency exchange rates telegram bot.

<img width="467" alt="Screenshot 2024-10-14 at 16 04 27" src="https://github.com/user-attachments/assets/3f0269c5-354d-4b61-a772-d3748db6636c">


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
