@echo off
rem Public release build (Nexus, GitHub Releases): dist\public\sekiro-together-v<version>.zip.
rem Differs from the test builds (build-dist.cmd / release.cmd):
rem - no `testsync` feature: no log streaming, no DLL push between players, and none of the
rem   testers' PowerShell launcher scripts embedded in the DLL (shipfiles.rs);
rem - local paths remapped (no C:\Users\<name> in panic messages or the PDB path);
rem - no Windows scripts in the zip: me3 opens sekiro-coop.me3 on a double-click (me3 launch -p).
rem   play.sh is for Linux / Steam Deck, where me3 is often not on PATH yet (~/.local/bin).
rem The text files come from ship-public\ and the licenses from the public repo's files here.
setlocal
set "VCVARS="
if exist "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if not defined VCVARS if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if not defined VCVARS (echo vcvars64.bat not found & exit /b 1)
call "%VCVARS%" >nul || exit /b 1
set "PATH=C:\Program Files\Rust stable MSVC 1.99\bin;%USERPROFILE%\.cargo\bin;%PATH%"
set "ROOT=%~dp0"
set "ROOT=%ROOT:~0,-1%"
set "CARGO_TARGET_DIR=%ROOT%\target-public"
rem A TOML array, not RUSTFLAGS: the project path has a space. The last matching remap wins.
cargo build --release -p sekiro_coop --no-default-features --features dev-native --config "build.rustflags=['--remap-path-prefix=%USERPROFILE%=~', '--remap-path-prefix=%ROOT%=.', '-Clink-arg=/PDBALTPATH:sekiro_coop.pdb']" || exit /b 1
rem `pub const LRM_VERSION: &str = "1.0.8-beta+lrm.1";` -> zip without '+'
for /f "tokens=2 delims==;" %%V in ('findstr /b /c:"pub const LRM_VERSION" "%ROOT%\crates\coop-dll\src\ui.rs"') do for %%A in (%%V) do set "LRM=%%~A"
if "%LRM%"=="" (echo LRM_VERSION not found & exit /b 1)
set "VERFILE=%LRM:+=-%"
set "S=%ROOT%\dist\public\sekiro-together"
if exist "%S%" rmdir /s /q "%S%"
mkdir "%S%\licenses"
copy /y "%CARGO_TARGET_DIR%\release\sekiro_coop.dll" "%S%\" >nul || exit /b 1
copy /y "%ROOT%\ship\sekiro-coop.me3" "%S%\" >nul || exit /b 1
copy /y "%ROOT%\ship-public\HOW-TO-PLAY.txt" "%S%\" >nul || exit /b 1
copy /y "%ROOT%\ship-public\play.sh" "%S%\" >nul || exit /b 1
copy /y "%ROOT%\ship-public\LICENSE.txt" "%S%\" >nul || exit /b 1
copy /y "%ROOT%\ship-public\licenses\*" "%S%\licenses\" >nul || exit /b 1
set "Z=sekiro-together-lrm-%VERFILE%.zip"
rem Windows tar (not Git's GNU tar on PATH) writes forward-slash paths; Compress-Archive's
rem backslashes break some unzippers.
pushd "%ROOT%\dist\public" || exit /b 1
if exist "%Z%" del "%Z%"
"%SystemRoot%\System32\tar.exe" -a -c -f "%Z%" sekiro-together || (popd & exit /b 1)
popd
echo valmis: dist\public\%Z%
