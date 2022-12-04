<!-- Allow this file to not have a first line heading -->
<!-- markdownlint-disable-file MD041 no-emphasis-as-heading -->

<!-- inline html -->
<!-- markdownlint-disable-file MD033 -->

<div align="center">

# `📦 Packer`

**Distribute Roblox games as standalone executables.**

`🚧 Packer is still being worked on. Among many other things, Windows is not currently supported. See below for details. 🚧`

![Packer Demonstration](.github/example.gif)
`Packer Example. Skips downloading client to speed up video.`

</div>

# About

Packer enables the distribution of Roblox experiences as standalone applications, allowing users without a prior installation of Roblox to play any Roblox experience.

At its core, a Packer application is just a custom client bootstrapper that downloads the latest Roblox client into the directory of the launcher. Games distributed with Packer aren't *actually* standalone, and they still use the Roblox client under the hood. However, the client is entirely portable and leaves no significant traces on the host machine. Unlike Roblox's default bootstrapper, Packer creates no new protocols or other lasting artefacts.

# What's TODO

This project is in the works!

- Windows support still needs to be added. Packer was developed on a Macbook, and I still need to pull everything down on my Windows desktop. Soontm!

- Proper authentication flows. This is a big one, and I need to take special care to ensure everything is safe. The most likely scenario is that special authentication processes will not be included in this public repository and will only be distributed privately. Currently, Packer won't run if there isn't existing Roblox authentication in the environment.

# License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

# Prior Art

- [`Roblox Studio Mod Manager`](https://github.com/MaximumADHD/Roblox-Studio-Mod-Manager) - A massive source of inspiration in designing the client bootstrapper. Much of the download-related code is borrowed from and inspired by Max's Mod Manager.
