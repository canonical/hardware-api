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

    __abstract__ = True
    id: Mapped[int] = mapped_column(primary_key=True, autoincrement=True)


class Vendor(Base):
    __tablename__ = "vendor"
    name: Mapped[str] = mapped_column(unique=True, nullable=False)
    # Relationships
    platforms: Mapped[list["Platform"]] = relationship(back_populates="vendor")
    bioses: Mapped[list["Bios"]] = relationship(back_populates="vendor")


class Platform(Base):
    __tablename__ = "platform"
    name: Mapped[str] = mapped_column(unique=True, nullable=False)
    # Relationships
    vendor_id: Mapped[int] = mapped_column(ForeignKey("vendor.id"), index=True)
    vendor: Mapped[Vendor] = relationship(back_populates="platforms")
    configurations: Mapped[list["Configuration"]] = relationship(
        back_populates="platform"
    )


class Configuration(Base):
    __tablename__ = "configuration"
    name: Mapped[str] = mapped_column(unique=True, nullable=False)
    # Relationships
    platform_id: Mapped[int] = mapped_column(ForeignKey("platform.id"), index=True)
    platform: Mapped[Platform] = relationship(back_populates="configurations")
    machines: Mapped[list["Machine"]] = relationship(back_populates="configuration")


class Machine(Base):
    __tablename__ = "machine"
    canonical_id: Mapped[str] = mapped_column(unique=True, nullable=False)
    # Relationships
    configuration_id: Mapped[int] = mapped_column(
        ForeignKey("configuration.id"), index=True
    )
    configuration: Mapped[Configuration] = relationship(back_populates="machines")
    certificates: Mapped[list["Certificate"]] = relationship(back_populates="machine")


class Certificate(Base):
    __tablename__ = "certificate"
    created_at: Mapped[datetime] = mapped_column(nullable=False)
    name: Mapped[str] = mapped_column(String(80), nullable=True)
    completed: Mapped[datetime] = mapped_column(nullable=True)
    # Relationships
    machine_id: Mapped[int] = mapped_column(
        ForeignKey("machine.id"), index=True, nullable=False
    )
    machine: Mapped[Machine] = relationship(back_populates="certificates")
    release_id: Mapped[int] = mapped_column(
        ForeignKey("release.id"), index=True, nullable=True
    )
    release: Mapped["Release"] = relationship(back_populates="certificates")
    reports: Mapped[list["Report"]] = relationship(back_populates="certificate")


class Release(Base):
    __tablename__ = "release"
    id: Mapped[int] = mapped_column(primary_key=True, autoincrement=True)
    codename: Mapped[str] = mapped_column(String(64), nullable=False)
    release: Mapped[str] = mapped_column(String(64), nullable=False)
    release_date: Mapped[date] = mapped_column(nullable=True)
    supported_until: Mapped[date] = mapped_column(nullable=True)
    i_version: Mapped[int] = mapped_column(nullable=True)
    # Relationships
    parent_id: Mapped[int] = mapped_column(
        ForeignKey("release.id"), index=True, nullable=True
    )
    parent: Mapped["Release"] = relationship(
        back_populates="children", remote_side=[id]
    )
    children: Mapped[list["Release"]] = relationship(back_populates="parent")
    certificates: Mapped[list["Certificate"]] = relationship(back_populates="release")


class Kernel(Base):
    __tablename__ = "kernel"
    name: Mapped[str] = mapped_column(nullable=False)
    version: Mapped[str] = mapped_column(nullable=False)
    signature: Mapped[str] = mapped_column(nullable=False)
    # Relationships
    reports: Mapped[list["Report"]] = relationship(back_populates="kernel")


class Bios(Base):
    __tablename__ = "bios"
    firmware_version: Mapped[str] = mapped_column(nullable=False)
    release_date: Mapped[date] = mapped_column(nullable=False)
    revision: Mapped[str] = mapped_column(nullable=False)
    version: Mapped[str] = mapped_column(nullable=False)
    # Relationships
    vendor_id: Mapped[int] = mapped_column(
        ForeignKey("vendor.id"), index=True, nullable=False
    )
    vendor: Mapped[Vendor] = relationship(back_populates="bioses")
    reports: Mapped[list["Report"]] = relationship(back_populates="bios")


class Report(Base):
    __tablename__ = "report"
    created_at: Mapped[datetime] = mapped_column(nullable=False)
    # Relationships
    kernel_id: Mapped[int] = mapped_column(
        ForeignKey("kernel.id"), index=True, nullable=False
    )
    kernel: Mapped[Kernel] = relationship(back_populates="reports")
    bios_id: Mapped[int] = mapped_column(
        ForeignKey("bios.id"), index=True, nullable=False
    )
    bios: Mapped[Bios] = relationship(back_populates="reports")
    certificate_id: Mapped[int] = mapped_column(
        ForeignKey("certificate.id"), index=True, nullable=False
    )
    certificate: Mapped[Certificate] = relationship(back_populates="reports")
