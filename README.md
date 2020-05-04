# callback

A utility for interacting with local macOS applications using [x-callback-url](http://x-callback-url.com).

## Compile from Source

```bash
$ git clone git@github.com:ozbe/x-callback-url.git
$ cd x-callback-url
$ make
```

## Install

```bash
$ echo "callback() { $(pwd)/callback.app/Contents/MacOS/callback \"\$@\" ;}" >> ~/.zshrc && source ~/.zshrc
```
## Uninstall

```bash
$ sed -i old '/^callback\(\)/d' ~/.zshrc && source ~/.zshrc
```

## Usage

Run callback with `callback -h` or `callback --help` to view the latest available flags, arguments, and
commands.

```text
callback 0.1.0
Interact with x-callback-url APIs

A utility for interacting with local macOS applications using x-callback-url (http://x-callback-url.com).

USAGE:
    callback <scheme> <action> [parameters]...

FLAGS:
    -h, --help       
            Prints help information

    -V, --version    
            Prints version information


ARGS:
    <scheme>           
            Scheme of target app
            
            Unique string identifier of the target app. Example: bear
    <action>           
            Name of action
            
            Action for target app to execute. Example: create
    <parameters>...    
            x-callback and action parameters
            
            Space delimited URL encoded x-callback-url parameters Example: title=My%20Note%20Title text=First%20line
```

Visit [x-callback-url Apps](http://x-callback-url.com/apps/) or the corresponding documentation for apps you have installed on your Mac to find x-callback-url APIs you can call with callback.

## Troubleshooting

* Double check the documentation for the callback url you are calling
* See if callback is running `$ ps -ax | grep callback.app` 
* Kill any instances of callback `$ killall callback` 

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.