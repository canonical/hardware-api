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
"""Store C3 API URLs"""

import os


C3_URL = os.environ.get("C3_URL", "https://certification.canonical.com")
PUBLIC_CERTIFICATES_URL = f"{C3_URL}/api/v2/public-certificates/"
PUBLIC_DEVICES_URL = f"{C3_URL}/api/v2/public-device-instances/"


def get_limit_offset(limit: int = 0) -> str:
    """Return string in format '?pagination=limitoffset&limit={limit}'"""
    return f"?pagination=limitoffset&limit={limit}"
