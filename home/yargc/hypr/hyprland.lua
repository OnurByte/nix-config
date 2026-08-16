local home = os.getenv("HOME")
package.path = package.path .. ";" .. home .. "/.config/hypr/?.lua;" .. home .. "/.config/hypr/?/init.lua"

require("vesper.appearance")
require("vesper.input")
require("vesper.autostart")
require("vesper.binds")
