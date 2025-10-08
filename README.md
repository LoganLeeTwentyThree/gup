# Gup
Gup is the package manager for the Halcyon programming language.
## Install gup
The binaries for gup can be found in the [releases](https://github.com/LoganLeeTwentyThree/gup/releases) section of this repository.
### Windows
Install `gup.exe` (Gup for windows) and put it somewhere you wont forget, like `C:\Program Files\gup`.
Edit your system environment PATH variable to include the directory that contains `gup.exe`.

### Linux 
Install `gup` (Gup for linux) and put it in `/bin`

## Use gup
### Config.toml
Gup looks for a .toml file named "Config.toml" in your project directory to help configure it. Config files have a number of required and optional fields:

\[package\]  
`name` : String  
`version` : String  

\[build\]  
*`infiles` : String Array  
*`outfile` : String  
`docfile` : String  

\[dependencies\]  
`dependencies` : Table<String, String>  

\* = required for compilation


### Commands
*  `check`  Validate the program without producing output
*  `build`  Compile and link the program
*  `run`    Compile, link, and execute the program
*  `init`   Initialize a config file and main file
*  `doc`    Creates documentation based off comment annotations
*  `tree`   Prints the dependency tree of the current project
*  `add`    Adds a dependency by URL or Path
*  `help`   Print help message or the help of the given subcommand(s)

### Docs
You can add documentation to a halcyon source file inside of comments.
The `doc` command looks for three fields: `@title:`, `@signature:`, and `@description:`, then formats them to markdown in an output file.
`@description` must end with an @ symbol as a delimiter.
`argdemo.hc` also contains example documentation comments.
