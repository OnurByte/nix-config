local home = os.getenv("HOME")
package.path = package.path .. ";" .. home .. "/.config/hypr/?.lua;" .. home .. "/.config/hypr/?/init.lua"

require("kraken.appearance")
require("kraken.input")
require("kraken.autostart")
require("kraken.binds")
