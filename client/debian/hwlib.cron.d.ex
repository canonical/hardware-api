#
# Regular cron jobs for the hwlib package
#
0 4	* * *	root	[ -x /usr/bin/hwlib_maintenance ] && /usr/bin/hwlib_maintenance
