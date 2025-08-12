# SamRewritten

<div align="center">
    <img src="/assets/icon_256.png" alt="SamRewritten Logo" width="128" />
    <br />
    <img src="/assets/screenshot1.png" alt="SamRewritten screenshot" width="320" />
    <br /><em>GTK version preview</em>
    <br />
    <img src="/assets/screenshot2.png" alt="SamRewritten screenshot" width="320" />
    <br /><em>Adwaita version preview</em>
    <br />
    <strong>A Steam Achievements Manager for Windows and Linux.</strong>
    <br />
    <a href="https://github.com/PaulCombal/SamRewritten/releases"><b>DOWNLOAD</b></a>
    <br />
    <em>
        This project and its contributors are not affiliated with Valve Corporation or Microsoft.<br />
        Steam and Windows are trademarks of their respective owners, Valve Corporation and Microsoft.
    </em>
</div>


## Thank You

SamRewritten is heavily inspired by other wonderful projects such as
* [Steam Achievements Manager by Gibbed](https://github.com/gibbed/SteamAchievementManager)
* [Samira by jsnli](https://github.com/jsnli/Samira)

Thanks to all the contributors of these amazing repositories, and also to
[the legacy version of this very project](https://github.com/PaulCombal/SamRewritten-legacy).

And most of all, thank you to all our awesome users and stargazers, giving us motivation to keep building.


## What is SamRewritten?

SamRewritten is a tool that allows you to unlock and lock achievements on your Steam account.
Additionally, some apps and games expose stats which can also be edited using this tool. Achievements do not have any
financial value, but they are highly desirable for bragging rights!


## Installation

Downloads are available in the [Releases tab](https://github.com/PaulCombal/SamRewritten/releases) for Windows (installer) and Linux (AppImage).


<details>
<summary>Windows Installation Instructions</summary>

The supported way to run SamRewritten on Windows is by using the installer. 
You can download the installer from the [Releases page](https://github.com/PaulCombal/SamRewritten/releases).
After running the installer and completing the installation, SamRewritten should appear in your Start menu.

If the installation does not complete as intended, please open an issue and provide as many details as possible, including your version of Windows.
</details>

<details>
<summary>Linux Installation Instructions</summary>

If your Linux distribution doesn't provide a way to install SamRewritten, you can use AppImages.
AppImages are self-contained executables designed to run independently of your Linux distribution.
AppImages for SamRewritten are available to download at the [Releases page](https://github.com/PaulCombal/SamRewritten/releases).
To run an AppImage, make sure you have permission to execute it. This can usually be set by right-clicking the file, navigating to permissions, and checking the "Allow executing file as program" box.
You should then be able to double-click the AppImage file to start SamRewritten.

If SamRewritten doesn't start, you can troubleshoot by running the AppImage from a terminal and examining the output:

```sh
./SamRewritten-gtk.AppImage
```

If the message in the console mentions Fuse or libfuse, you might need to install it and try again:

```sh
sudo apt install libfuse2 # Example for Ubuntu/Debian
```

If the error persists, please open an issue including your Linux distribution and version, as well as the console output.
</details>

> **Note**
> For Arch Linux and derivatives, you can install SamRewritten with yay:
>
> ```sh
> yay -S samrewritten-git
> ```


<!--
Additionally, Snap users can install SamRewritten using the App store or with the following command:
```sh
snap install samrewritten
```
-->


## Features

- Lock and unlock select achievements with a single click
- Edit statistics instantly
- Schedule achievement unlocking over a set period of time


## Limitations

⚠️ On Linux, this tool is **only** compatible with:
- Snap installations of Steam
- Ubuntu/Debian multiarch installations with apt
- Distribution installations that use the Steam runtime (Gentoo, Arch, `~/.steam/root` exists)

If you wish to see your distribution supported, please open an issue.

> **Tip**
> Flatpak support poses a considerable challenge. If you or someone you know with knowledge of Flatpak internals can offer to help, please reach out!


## End User Agreements

This software serves as a Proof-of-Concept. Users are responsible for their actions using this tool.
By using this tool, you agree that you are solely responsible for the management of your Steam account. None of the
contributors can be held responsible for the actions and their repercussions you have taken using this tool.

Using this tool on multiplayer games is highly discouraged.
