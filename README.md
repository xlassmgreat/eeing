# eeing
Slimmed down chess engine wrapper for native messaging

Eeing lets browser (Firefox only as of now) plugins to access Chess engine analysis through Native Messaging. It DOES NOT let the plugin make any configuration for the engine, or even decide how long to analyse. Such decisions are only given to the user.

## Configuration
The configuration is done via a TOML file called `config.toml` which is expected to be in the directly from where the eeing binary is run, usually the same directory where the binary actually exist. Just put it in the same directory as the executable. There are a few options that you can set:

|Option            | Value                            | Description                                             |
|----------------- | -------------------------------- | ------------------------------------------------------- |
|engine            | table                            | describes engine behaviour
|  command         | string                           | command to execute the engine                           |
|  args            | optional; array of strings       | arguments to give to the engine                         |
|  config          | table                            | [UCI option] = [value] table to set UCI options; use "_"|
|                  |                                  | if the option has a space in it                         |
|movetime          | integer                          | time to spend per move                                  |
|depth             | integer                          | depth per move                                          |
|node              | integer                          | nodes per move                                          |

This is a basic example which you can use:

```toml
movetime = 1000

[engine]
command = "stockfish"

[engine.config]
hash = 128
```
Make a new file in the directory called `config.toml` and paste the above to get started.

## Installation
Grab one of the [releases](https://github.com/xlassmgreat/eeing/releases) and run the installation file `install.*` from the directory which contains the eing binary (`install.sh` for Linux, `install.bat` for Windows). All it does in Windows is create a registry, so each time you move the binary, you'll have to recreate the registry. And on Linux, it takes the current path and puts it into app maifest at `~/.mozilla/native-messaging-hosts/eeing.json`. You can also do it manually of course. What this means is that on both platforms, each time you relocate the binary, you'll have to rerun the install. Beware that on Windows, rerunning the installation file DOES NOT erase the previous registry.
