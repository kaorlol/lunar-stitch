# Lunar Stitch

## Description

Lunar Stitch is a Roblox luau bundler that allows you to bundle your code into a single file. This is useful because it allows you to make your code more modular and easier to manage. It also allows you to easily share your code with others. Lunar Stitch is designed to be easy to use and flexible. Lunar Stitch is open source and free to use. It is licensed under the MIT license.

## Installation

Install [binary](https://github.com/kaorlol/lunar-stitch/releases/latest) for your operating system.

#### Usage:

```sh
lunar-stitch.exe [OPTIONS]
```

##### Example:

```sh
lunar-stitch.exe -r src -i main.lua -o bundled.lua
```

### Options:

```
-r, --root <ROOT> The root directory to use [default: .]
-i, --input <INPUT> The input file to read from [default: main.lua]
-o, --output <OUTPUT> The output file to write to [default: bundled.lua]
-m, --minify Whether to minify the output
-b, --beautify Whether to beautify the output
-h, --help Print help
-V, --version Print version
```

## License

MIT License

Copyright (c) 2024 kaoru

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
