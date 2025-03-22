#  Copyright 2023-2025, [rust@localhost] $ (@3532340532)
# 
#  This file is part of EfficientScheduler.
# 
#  EfficientScheduler is free software: you can redistribute it and/or modify it under
#  the terms of the GNU General Public License as published by the Free
#  Software Foundation, either version 3 of the License, or (at your option)
#  any later version.
# 
#  EfficientScheduler is distributed in the hope that it will be useful, but WITHOUT ANY
#  WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
#  FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
#  details.
# 
#  You should have received a copy of the GNU General Public License along
#  with EfficientScheduler. If not, see <https:://www.gnu.org/licenses/>.

local_print() {
	if [ $LOCALE = zh-CN ]; then
		ui_print "$1"
	else
		ui_print "$2"
	fi
}

local_print "一个高效的调度" "An efficient scheduling"
local_print "- 检查中..." "- Under inspection ..."
if [ $ARCH != arm64 ]; then
	local_print "设备不支持, 非arm64设备 !" "Only for arm64 device !"
	abort
elif [ $API -le 30 ]; then
	local_print "系统版本过低, 需要安卓12及以上的系统版本版本 !" "Required A12+ !"
	abort
elif uname -r | awk -F. '{if ($1 < 5 || ($1 == 5 && $2 < 8)) exit 0; else exit 1}'; then
	local_print "内核版本过低，需要5.8或以上 !" "The kernel version is too low. Requires 5.8+ !"
	abort
fi
set_perm_recursive $MODPATH 0 0 0755 0644
set_perm $MODPATH/EfficientScheduler 0 0 0755