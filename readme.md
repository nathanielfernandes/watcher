# watcher

a simple discord presence watcher that can be listened to via server sent events

**example usage on my website:**
![example](https://cdn.discordapp.com/attachments/1143578217911951491/1147170397180153907/image.png)

## Usage

join the discord server https://discord.gg/TnJPMdMDQZ

**if you want your live activity:**
`https://watcher.ncp.nathanferns.xyz/live-activity/<your discord id>`

**if you want grab just your current status:**
`https://watcher.ncp.nathanferns.xyz/activity/<your discord id>`

> Typescript definition are available in the repo under `./ts/activity.ts`

## Running

export the following environment variables:

```bash
TOKEN=<discord bot token>
SERVER=<discord server id>
```

```
cargo run --release
```
