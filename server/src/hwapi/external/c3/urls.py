# Copyright 2024 Canonical Ltd.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License version
# 3, as published by the Free Software Foundation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# Written by:
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
"""Store C3 API URLs"""

import os

C3_URL = os.environ.get("C3_URL", "https://certification.canonical.com")
PUBLIC_CERTIFICATES_URL = f"{C3_URL}/api/v2/public-certificates/"
PUBLIC_DEVICES_URL = f"{C3_URL}/api/v2/public-devices/"
CPU_IDS_URL = f"{C3_URL}/api/v2/cpuids/"


def get_limit_offset(limit: int = 0) -> str:
    """Return string in format '?pagination=limitoffset&limit={limit}'"""
    return f"?pagination=limitoffset&limit={limit}"
