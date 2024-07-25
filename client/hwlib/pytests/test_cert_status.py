# Copyright 2024 Canonical Ltd.
# All rights reserved.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# Written by:
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>

from hwlib import send_certification_request


def test_send_certification_request():
    """Verify that we can use the library in the python code"""
    assert callable(send_certification_request)
    url = "http://127.0.0.1:8000"
    result = send_certification_request(
        url,
        "tests/test_data/smbios_entry_point",
        "tests/test_data/DMI",
        "tests/test_data/cpuinfo",
        "tests/test_data/cpuinfo_max_freq",
        "tests/test_data/device-tree",
        "tests/test_data/version",
    )
    assert isinstance(result, dict)
