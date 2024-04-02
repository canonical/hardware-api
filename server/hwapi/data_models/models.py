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


from datetime import date, datetime

from sqlalchemy import ForeignKey, String
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column
from sqlalchemy.orm import relationship


class Base(DeclarativeBase):
    """Base model for all the models"""

    id: Mapped[int] = mapped_column(primary_key=True, autoincrement=True)


class Vendor(Base):
    __tablename__ = "vendors"
    name: Mapped[str] = mapped_column(unique=True, nullable=False)


class Platform(Base):
    __tablename__ = "platforms"
    name: Mapped[str] = mapped_column(unique=True, nullable=False)
    vendor_id: Mapped[int] = mapped_column(ForeignKey("vendors.id"))
    vendor: Mapped[Vendor] = relationship(foreign_keys=[vendor_id])


class Configuration(Base):
    __tablename__ = "configurations"
    name: Mapped[str] = mapped_column(unique=True, nullable=False)
    platform_id: Mapped[int] = mapped_column(ForeignKey("platforms.id"))
    platform: Mapped[Platform] = relationship(foreign_keys=[platform_id])


class Machine(Base):
    __tablename__ = "machines"
    canonical_id: Mapped[str] = mapped_column(unique=True, nullable=False)
    configuration_id: Mapped[int] = mapped_column(ForeignKey("configurations.id"))
    configuration: Mapped[Configuration] = relationship(foreign_keys=[configuration_id])


class Certificate(Base):
    __tablename__ = "certificates"
    machine_id: Mapped[int] = mapped_column(ForeignKey("machines.id"), nullable=False)
    machine: Mapped[Machine] = relationship(foreign_keys=[machine_id])
    created_at: Mapped[datetime] = mapped_column(nullable=False)
    name: Mapped[str] = mapped_column(String(80), nullable=True)
    completed: Mapped[datetime] = mapped_column(nullable=True)
    release_id: Mapped[int] = mapped_column(ForeignKey("releases.id"), nullable=True)
    release: Mapped["Release"] = relationship(foreign_keys=[release_id])


class Release(Base):
    __tablename__ = "releases"
    codename: Mapped[str] = mapped_column(String(64), nullable=False)
    release: Mapped[str] = mapped_column(String(64), nullable=False)
    release_date: Mapped[date] = mapped_column(nullable=True)
    supported_until: Mapped[date] = mapped_column(nullable=True)
    i_version: Mapped[int] = mapped_column(nullable=True)
    parent_id: Mapped[int] = mapped_column(ForeignKey("releases.id"), nullable=True)
    parent: Mapped["Release"] = relationship(foreign_keys=[parent_id])
    url_template: Mapped[str] = mapped_column(String(255), nullable=True)


class Kernel(Base):
    __tablename__ = "kernels"
    name: Mapped[str] = mapped_column(nullable=False)
    version: Mapped[str] = mapped_column(nullable=False)
    signature: Mapped[str] = mapped_column(nullable=False)


class Bios(Base):
    __tablename__ = "bios"
    firmware_version: Mapped[str] = mapped_column(nullable=False)
    release_date: Mapped[date] = mapped_column(nullable=False)
    revision: Mapped[str] = mapped_column(nullable=False)
    vendor_id: Mapped[int] = mapped_column(ForeignKey("vendors.id"), nullable=False)
    version: Mapped[str] = mapped_column(nullable=False)
    vendor: Mapped[Vendor] = relationship(foreign_keys=[vendor_id])


class Report(Base):
    __tablename__ = "reports"
    created_at: Mapped[datetime] = mapped_column(nullable=False)
    kernel_id: Mapped[int] = mapped_column(ForeignKey("kernels.id"), nullable=False)
    kernel: Mapped[Kernel] = relationship(foreign_keys=[kernel_id])
    bios_id: Mapped[int] = mapped_column(ForeignKey("bios.id"), nullable=False)
    bios: Mapped[Bios] = relationship(foreign_keys=[bios_id])
    certificate_id: Mapped[int] = mapped_column(
        ForeignKey("certificates.id"), nullable=False
    )
    certificate: Mapped[Certificate] = relationship(foreign_keys=[certificate_id])
