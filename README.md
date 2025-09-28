# Vintakon

![](data/app_icon_256.png)

Vintakon is an special case of Exiftool UI, specifically designed to create profiles for lenses that don't provide exif
information and apply these profiles to multiple files at the same time.

Vintakon uses tested and proven [exiftool](https://exiftool.org/) for reading and writing exif information and supports
most of the image file formats that are supported by exiftool.

Vintakon name doesn't mean anything. It's just a play on word "vintage" and "-kon" suffix, which sounds like a
vintage lens name.

## Using

`exiftool` must be installed. Use your package manager to install it on Linux. On MacOS you can install it using Homebrew.

On MacOS and Windows you need to set path to exiftool executable in settings. On linux it should work out of the box.

## Limitations

exiftool is unable to overwrite all tags. For example, `FocalLength` in Sony a7IV .ARW files
is not stored in standard EXIF tag 920A, but instead in maker notes. Exiftool doesn't allow to
alter maker notes to avoid potential conflicts with vendor software. This means that if you apply
lens profile with a Focal length to .ARW file, the focal length tag will not be updated.
