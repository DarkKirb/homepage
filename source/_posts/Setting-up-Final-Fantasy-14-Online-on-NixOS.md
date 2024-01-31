---
title: 'Setting up Final Fantasy 14: Online on NixOS'
date: 2024-01-31 13:44
categories:
    - Games
    - Final Fantasy
    - Final Fantasy 14
tags:
    - NixOS
    - Final Fantasy
    - Final Fantasy 14
---

Final Fantasy 14 is an MMORPG that is fully playable on Linux with only minor bugs and that I have been playing for a while now. This post will describe my personal setup of it.

Parts of this will also apply to non-NixOS systems

<!-- more -->

## The Simple Way (Flatpak)

If you just want to play the game and don’t care about external programs like [Advanced Combat Tracker](https://github.com/FFXIV-ACT/setup-guide) or [FFXIV Teamcraft](https://ffxivteamcraft.com/), you can just directly install the [XIVLauncher Flatpak](https://flathub.org/apps/dev.goats.xivlauncher).

One note that if you register via Steam, you need to have steam running when starting the game, otherwise logging in will not work. I don’t know if the steam flatpak would work with this.

## The Nix Way

The [XIVLauncher](https://mynixos.com/nixpkgs/package/xivlauncher) I mentioned before is packaged on NixOS, but there you have to bring your own working wine, as the default one won’t work.

### The Wine to use

I personally use a patched version of wine-ge for FFXIV Teamcraft, as that application doesn’t work as administrator. Of course there are a few other options available.

The wine that is shipped with xivlauncher is based on the tkg patchset with [a few extra patches](https://github.com/Frogging-Family/wine-tkg-git/compare/master...goatcorp:wine-xiv-git:master) that I personally haven’t needed.

#### standard wine-staging

Haven’t tried this. May work, may not work.

#### wine-tkg/wine-ge

wine-tkg is going to be able to run the game, however it is not currently packaged on nixpkgs. However, You can find it in [Fufexan’s nix-gaming repo](https://github.com/fufexan/nix-gaming).

I’m adding wine-ge to here too since it is also available from the same repo and based on the tkg patchset.

#### Patching your wine

As mentioned before, some 3rd party applications may benefit from extra patches. [I personally just use the “server-default_integrity” patch set](https://github.com/DarkKirb/nixos-config/blob/main/config/games/default.nix), however this is where you would add other patches, including the ones that the XIVLauncher ships with.

### Installing the Game

Look through all of the options that XIVLauncher has. The most important ones are the following one:

- Use steam service (on the main page) if your service account is tied to your steam account
- `Settings → Game → Free Trial Account`: if you are using a free trial account (duh)
- `Settings → Wine → Wine Version`: set it to custom.
- `Settings → Wine → Wine Binary Path`: Set it to the value of `dirname $(which wine)`.

At this point just follow the login flow. It will automatically download and install the game and configure the wineprefix for the game’s use.

For the following steps you need to set the following configuration variables in your shell depending on what your settings are in XIVLauncher.

- `WINEPREFIX` is set to the wine prefix set in the option, by default `~/.xlcore/wineprefix`
- `WINEESYNC` is set to `1` if you enabled ESync
- `WINEFSYNC` is set to `1` if you enabled FSync

You then need to install the `dxvk` and `dotnet48` winetricks packages. Especially the dotnet installation can break the wineprefix, and you may need to remove it and start the game up again.

If you are successful with installing both, you then need to go into `winecfg` and change the windows version to something recent like Windows 10. [^1]

[^1]: This step might no longer be necessary, but these steps used to result in wine’s version being set to Windows Server 2003.

Afterwards, try starting the game again. It should still work.

Now you should just be able to install FFXIV Teamcraft and ACT normally. In the FFXIV ACT Plugin settings you need to “Inject and use Deucalion for network data”. Otherwise it will not be able to pick up the logs automatically.

### Starting the game

Might seem easy but it’s not. You need to make sure that the game starts in the wineprefix first. Close all other applications in the wineprefix before launching the game, otherwise Dalamud will not work.

### Starting third party applications

Similarly looks simple, but for electron-based 3rd party tools you need to pass the command line argument `--disable-sandbox`.

The .desktop entries that i use look as follows, they don’t have the correct icon yet.

```ini
[Desktop Entry]
Comment=
Exec=env WINEPREFIX=/home/darkkirb/.xlcore/wineprefix WINEFSYNC=1 wine '/home/darkkirb/.xlcore/wineprefix/drive_c/users/darkkirb/AppData/Local/ffxiv-teamcraft/FFXIV Teamcraft.exe' --no-sandbox
Name=FFXIV Teamcraft
NoDisplay=false
Path=
StartupNotify=true
Terminal=false
TerminalOptions=
Type=Application
X-KDE-SubstituteUID=false
X-KDE-Username=
```

```ini
[Desktop Entry]
Comment=
Exec=env WINEPREFIX=/home/darkkirb/.xlcore/wineprefix WINEFSYNC=1 wine '/home/darkkirb/.xlcore/wineprefix/drive_c/Program Files (x86)/Advanced Combat Tracker/Advanced Combat Tracker.exe' --no-sandbox
Name=FFXIV ACT
NoDisplay=false
Path=
StartupNotify=true
Terminal=false
TerminalOptions=
Type=Application
X-KDE-SubstituteUID=false
X-KDE-Username=
```

and finally

```ini
[Desktop Entry]
Comment=
Exec=env WINEPREFIX=/home/darkkirb/.xlcore/wineprefix WINEFSYNC=1 wine '/home/darkkirb/.xlcore/wineprefix/drive_c/Program Files (x86)/Advanced Combat Tracker/Advanced Combat Tracker.exe' --no-sandbox
Name=FFXIV ACT
NoDisplay=false
Path=
StartupNotify=true
Terminal=false
TerminalOptions=
Type=Application
X-KDE-SubstituteUID=false
X-KDE-Username=
```