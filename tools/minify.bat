@echo off

for %%f in (..\assets\src\*.js) do (
    echo Minificando %%~nxf
    terser "%%f" --compress "passes=3,drop_console=true,drop_debugger=true" --mangle --toplevel --output "..\assets\dist\%%~nf.min.js"
)
pause
