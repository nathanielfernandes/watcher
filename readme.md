# watcher

a simple discord presence watcher that can be listend to via server sent events

## Usage

join the discord server https://discord.gg/TnJPMdMDQZ

**if you want your live acitivy:**
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
