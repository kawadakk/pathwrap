# pathwrap

A wrapper program to substitute long paths in command-line arguments to circumvent issues caused by Windows path limits, such as MinGW gcc refusing to open an input file ([rust-lang/rust#48737](https://github.com/rust-lang/rust/issues/48737)).

```console
$ gcc -c aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/foo.c
cc1.exe: fatal error: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/foo.c: No such file or directory
compilation terminated.

$ pathwrap gcc -c aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/foo.c
```


## Prerequisites

- rustup to compile pathwrap

- A recent version of Windows
    - You must enable [Developer Mode](https://learn.microsoft.com/en-us/windows/uwp/get-started/enable-your-device-for-development) to allow [creation](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createsymboliclinkw) of symbolic links by an unprivileged user account.

## Usage

This program can operate in two modes depending on how the command is provided:

### argv0 mode

```console
$ gcc-pathwrap foo/bar.o -o foo
```

If the executable name (`argv[0]`) includes a suffix `-pathwrap`, pathwrap will execute the command specified by the rest of the name.

This mode offers a better compatibility, but requires you to place a copy or [link](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/new-item?view=powershell-7.4#example-7-create-a-symbolic-link-to-a-file-or-folder) to pathwrap's executable for each wrapped command.


### argv1 mode

```console
$ pathwrap gcc foo/bar.o -o foo
```

If the executable name (`argv[0]`) does not meet the condition for argv0 mode, pathwrap will operate in argv1 mode.
In this mode, the first command-line argument (`argv[1]`) specifies the command to execute.


## Principles of Operation

pathwrap works by checking every command-line argument that looks like a path and replacing its parent path with a symbolic link [created](https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_file.html) inside a [temporary directory](https://docs.rs/tempfile/3.10.1/tempfile/fn.tempdir.html).


## License

This program is licensed under [the GNU General Public License version 3](https://www.gnu.org/licenses/gpl-3.0.en.html).

> **16. Limitation of Liability.**
> 
> IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN WRITING WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MODIFIES AND/OR CONVEYS THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES, INCLUDING ANY GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES ARISING OUT OF THE USE OR INABILITY TO USE THE PROGRAM (INCLUDING BUT NOT LIMITED TO LOSS OF DATA OR DATA BEING RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD PARTIES OR A FAILURE OF THE PROGRAM TO OPERATE WITH ANY OTHER PROGRAMS), EVEN IF SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.
