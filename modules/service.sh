#!/system/bin/sh
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

MODDIR=${0%/*}

sh $MODDIR/vtools/init_vtools.sh $(realpath $MODDIR/module.prop)

RUST_BACKTRACE=1 nohup $MODDIR/EfficientScheduler > $MODDIR/run.log 2>&1 &