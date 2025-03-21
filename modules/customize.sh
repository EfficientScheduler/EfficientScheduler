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

ui_print "专为萌新做的调度"
ui_print "配置文件在/data/adb/modules/EfficientScheduler/config.toml"
ui_print "/data/adb/modules/EfficientScheduler/config/目录下的文件是自定义频率等文件，文件必须以toml结尾"
set_perm_recursive $MODPATH 0 0 0755 0644
set_perm $MODPATH/EfficientScheduler 0 0 0755