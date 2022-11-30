# eeing
Slimmed down chess engine wrapper for native messaging

Eeing lets browser (Firefox only as of now) plugins to access Chess engine analysis through Native Messaging. It DOES NOT let the plugin make any configuration for the engine, or even decide how long to analyse. Such decisions are only given to the user.

## Configuration
The configuration is done via a TOML file called `config.toml` which is expected to be in the directly from where the eeing binary is run, usually the same directory where the binary actually exist. Just put it in the same directory as the executable; it should already be there though. There are a few options that you can set:

|Option            | Value                            | Description                                             |
|----------------- | -------------------------------- | ------------------------------------------------------- |
|random_moves      | boolean value; true or false     | If set to true, it plays random moves.                  |
|engine_command    | path to an engine                | Should be set to the path of the desired engine.        |
|hash              | positive whole integer value     | The hash size to be used by the engine.                 |
|threads           | positive whole integer value     | The number of threads to be used by the engine.         |
|engine_debug_file  | path                             | The debug log file path to be used by the engine.        |
|limit             | time or depth as a nested option | The limit to be used for calculating moves.             |
|                  | time                             | The time given for each move in miliseconds.            |
|                  | depth                            | The depth given for each move.                          |
|                  |                                  | Note: Only one of time or depth can be provided         |

## Installation
Grab one of the [releases](https://github.com/xlassmgreat/eeing/releases) and run the installation file `install.*` from the directory which contains the eing binary (`install.sh` for Linux, `install.bat` for Windows). All it does in Windows is create a registry, so each time you move the binary, you'll have to recreate the registry. And on Linux, it takes the current path and puts it into app maifest at `~/.mozilla/native-messaging-hosts/eeing.json`. You can also do it manually of course. What this means is that on both platforms, each time you relocate the binary, you'll have to rerun the install. Beware that on Windows, rerunning the installation file DOES NOT erase the previous registry.
