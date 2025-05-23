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


from datetime import date, datetime

from sqlalchemy import (
    ForeignKey,
    String,
    UniqueConstraint,
    Table,
    Column,
    Integer,
    Enum,
)
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column
from sqlalchemy.orm import relationship

from .enums import BusType, DeviceCategory


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
    devices: Mapped[list["Device"]] = relationship(back_populates="vendor")


class Platform(Base):
    __tablename__ = "platform"
    name: Mapped[str] = mapped_column(nullable=False)
    # Relationships
    vendor_id: Mapped[int] = mapped_column(ForeignKey("vendor.id"), index=True)
    vendor: Mapped[Vendor] = relationship(back_populates="platforms")
    configurations: Mapped[list["Configuration"]] = relationship(
        back_populates="platform"
    )

    __table_args__ = (UniqueConstraint("name", "vendor_id"),)


class Configuration(Base):
    __tablename__ = "configuration"
    name: Mapped[str] = mapped_column(nullable=False)
    # Relationships
    platform_id: Mapped[int] = mapped_column(ForeignKey("platform.id"), index=True)
    platform: Mapped[Platform] = relationship(back_populates="configurations")
    machines: Mapped[list["Machine"]] = relationship(back_populates="configuration")

    __table_args__ = (UniqueConstraint("name", "platform_id"),)


class Machine(Base):
    __tablename__ = "machine"
    canonical_id: Mapped[str] = mapped_column(unique=True, nullable=False, index=True)
    # Relationships
    configuration_id: Mapped[int] = mapped_column(
        ForeignKey("configuration.id"), index=True
    )
    configuration: Mapped[Configuration] = relationship(back_populates="machines")
    certificates: Mapped[list["Certificate"]] = relationship(back_populates="machine")


class Certificate(Base):
    __tablename__ = "certificate"
    name: Mapped[str] = mapped_column(
        String(80), unique=True, nullable=False, index=True
    )
    created_at: Mapped[datetime] = mapped_column(nullable=False)
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
    codename: Mapped[str] = mapped_column(String(64), nullable=False)
    release: Mapped[str] = mapped_column(String(64), nullable=False)
    release_date: Mapped[date] = mapped_column(nullable=True)
    supported_until: Mapped[date] = mapped_column(nullable=True)
    i_version: Mapped[int] = mapped_column(nullable=True)
    # Relationships
    certificates: Mapped[list["Certificate"]] = relationship(back_populates="release")


class Kernel(Base):
    __tablename__ = "kernel"
    name: Mapped[str] = mapped_column(nullable=True)
    version: Mapped[str] = mapped_column(nullable=False)
    signature: Mapped[str] = mapped_column(nullable=True)
    # Relationships
    reports: Mapped[list["Report"]] = relationship(back_populates="kernel")


class Bios(Base):
    __tablename__ = "bios"
    release_date: Mapped[date] = mapped_column(nullable=True)
    firmware_revision: Mapped[str] = mapped_column(nullable=True)
    revision: Mapped[str] = mapped_column(nullable=True)
    version: Mapped[str] = mapped_column(nullable=False)
    # Relationships
    vendor_id: Mapped[int] = mapped_column(
        ForeignKey("vendor.id"), index=True, nullable=False
    )
    vendor: Mapped[Vendor] = relationship(back_populates="bioses")
    reports: Mapped[list["Report"]] = relationship(back_populates="bios")


device_report_association = Table(
    "device_report_association",
    Base.metadata,
    Column("device_id", Integer, ForeignKey("device.id"), primary_key=True),
    Column("report_id", Integer, ForeignKey("report.id"), primary_key=True),
)


class Report(Base):
    __tablename__ = "report"
    architecture: Mapped[str] = mapped_column(nullable=False)
    # Relationships
    kernel_id: Mapped[int] = mapped_column(
        ForeignKey("kernel.id"), index=True, nullable=True
    )
    kernel: Mapped[Kernel] = relationship(back_populates="reports")
    bios_id: Mapped[int] = mapped_column(
        ForeignKey("bios.id"), index=True, nullable=True
    )
    bios: Mapped[Bios] = relationship(back_populates="reports")
    certificate_id: Mapped[int] = mapped_column(
        ForeignKey("certificate.id"), index=True, nullable=False
    )
    certificate: Mapped[Certificate] = relationship(back_populates="reports")
    devices = relationship(
        "Device", secondary=device_report_association, back_populates="reports"
    )


class Device(Base):
    __tablename__ = "device"
    identifier: Mapped[str] = mapped_column(nullable=False, index=True)
    name: Mapped[str] = mapped_column(nullable=False, index=True)
    subproduct_name: Mapped[str] = mapped_column(nullable=False)
    device_type: Mapped[str] = mapped_column(nullable=False)
    bus: Mapped[str] = mapped_column(Enum(BusType), nullable=False)
    version: Mapped[str] = mapped_column(String(10), nullable=False)
    subsystem: Mapped[str] = mapped_column(nullable=False)
    category: Mapped[str] = mapped_column(Enum(DeviceCategory), nullable=False)
    codename: Mapped[str] = mapped_column(String(40), nullable=False)
    # Relationships
    vendor_id: Mapped[int] = mapped_column(ForeignKey("vendor.id"), index=True)
    vendor = relationship("Vendor", back_populates="devices")
    reports = relationship(
        "Report", secondary=device_report_association, back_populates="devices"
    )


class CpuId(Base):
    __tablename__ = "cpu_id"
    id_pattern: Mapped[str] = mapped_column(nullable=False)
    codename: Mapped[str] = mapped_column(nullable=False)
