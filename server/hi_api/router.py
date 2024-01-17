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


from fastapi import APIRouter
from .endpoints import certification, hardware, submission

router = APIRouter()
router.include_router(certification.router, prefix="/v1/certification", tags=["certification"])
router.include_router(hardware.router, prefix="/v1/hardware", tags=["hardware"])
router.include_router(submission.router, prefix="/v1/submission", tags=["submission"])


@router.get("/")
def root():
    return "Hardware Information API (HiAPI) server"
