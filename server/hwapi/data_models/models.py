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


from sqlalchemy import (
    Column,
    ForeignKey,
    Integer,
    String,
    DateTime,
    Date,
)
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship


Base = declarative_base()


class Vendor(Base):
    __tablename__ = "vendors"
    id = Column(Integer, primary_key=True)
    name = Column(String, unique=True, nullable=False)


class Platform(Base):
    __tablename__ = "platforms"
    id = Column(Integer, primary_key=True)
    name = Column(String, unique=True, nullable=False)
    vendor_id = Column(Integer, ForeignKey("vendors.id"))
    vendor = relationship("Vendor")


class Configuration(Base):
    __tablename__ = "configurations"
    id = Column(Integer, primary_key=True)
    name = Column(String, unique=True, nullable=False)
    platform_id = Column(Integer, ForeignKey("platforms.id"))
    platform = relationship("Platform")


class Machine(Base):
    __tablename__ = "machines"
    id = Column(Integer, primary_key=True)
    canonical_id = Column(String, unique=True, nullable=False)
    configuration_id = Column(Integer, ForeignKey("configurations.id"))
    configuration = relationship("Configuration")


class Certificate(Base):
    __tablename__ = "certificates"
    id = Column(Integer, primary_key=True)
    hardware_id = Column(Integer, ForeignKey("machines.id"), nullable=False)
    hardware = relationship("Machine")
    created_at = Column(DateTime, nullable=False)
    name = Column(String(80), nullable=True)
    completed = Column(DateTime, nullable=True)
    release_id = Column(Integer, ForeignKey("releases.id"), nullable=True)
    release = relationship("Release")


class Release(Base):
    __tablename__ = "releases"
    id = Column(Integer, primary_key=True)
    codename = Column(String(64), nullable=False)
    release = Column(String(64), nullable=False)
    release_date = Column(Date, nullable=True)
    supported_until = Column(Date, nullable=True)
    i_version = Column(Integer, nullable=True)
    parent_id = Column(Integer, ForeignKey("releases.id"), nullable=True)
    parent = relationship("Release", remote_side=[id])
    url_template = Column(String(255), nullable=True)


class Kernel(Base):
    __tablename__ = "kernels"
    id = Column(Integer, primary_key=True)
    name = Column(String, nullable=False)
    version = Column(String, nullable=False)
    signature = Column(String, nullable=False)


class Bios(Base):
    __tablename__ = "bios"
    id = Column(Integer, primary_key=True)
    firmware_version = Column(String, nullable=False)
    release_date = Column(Date, nullable=False)
    revision = Column(String, nullable=False)
    vendor_id = Column(Integer, ForeignKey("vendors.id"), nullable=False)
    version = Column(String, nullable=False)
    vendor = relationship("Vendor")


class Report(Base):
    __tablename__ = "reports"
    id = Column(Integer, primary_key=True)
    created_at = Column(DateTime, nullable=False)
    kernel_id = Column(Integer, ForeignKey("kernels.id"), nullable=False)
    bios_id = Column(Integer, ForeignKey("bios.id"), nullable=False)
    kernel = relationship("Kernel")
    bios = relationship("Bios")
    certificate_id = Column(Integer, ForeignKey("certificates.id"), nullable=False)
    certificate = relationship("Certificate")
