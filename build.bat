rustup run stable-i686-pc-windows-gnu cargo build --release

set EXT_PATH=C:\Users\daslocom\Documents\GameMaker\Projects\LD49_cornwall_2.gmx\extensions
set DROP_PATH=C:\Users\daslocom\ad_libber\target\release

set GMX_XML_NAME=ad_libber.xml
set GMX_NAME=ad_libber.extension.gmx
set DLL_NAME=ad_libber.dll
set TMP_PATH=C:\users\daslocom\tmp

del "%EXT_PATH%\%GMX_NAME%"
del "%EXT_PATH%\ad_libber\%DLL_NAME%"
copy "%DROP_PATH%\%DLL_NAME%" "%EXT_PATH%\ad_libber"
REM move "%EXT_PATH%\rope_lib\windows_lib.dll" "%EXT_PATH%\wrope_lib\orld_generators.dll"
move "%TMP_PATH%\%GMX_XML_NAME%" "%TMP_PATH%\%GMX_NAME%"
copy "%TMP_PATH%\%GMX_NAME%" "%EXT_PATH%"
REM move "%EXT_PATH%\rope_lib.xml" "%EXT_PATH%\rope_lib.extension.gmx"