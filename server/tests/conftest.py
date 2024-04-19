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
#        Omar Selo <omar.selo@canonical.com

from os import environ
import pytest

from fastapi.testclient import TestClient
from sqlalchemy import Engine, create_engine
from sqlalchemy.orm import Session, sessionmaker
from sqlalchemy_utils import (  # type: ignore
    create_database,
    drop_database,
)

from hwapi.data_models.setup import get_db
from hwapi.data_models.models import Base
from hwapi.main import app
from tests.data_generator import DataGenerator


@pytest.fixture(scope="session")
def db_url():
    """
    Retrieves the database url from the environment variable TEST_DB_URL
    or creates a new database and returns the url
    """
    db_url = environ.get("DB_URL")
    if db_url:
        yield db_url
    else:
        db_url = "sqlite://"
        create_database(db_url)

        yield db_url

        drop_database(db_url)


@pytest.fixture(scope="session")
def db_engine(db_url: str):
    engine = create_engine(db_url, connect_args={"check_same_thread": False})
    Base.metadata.create_all(engine)
    yield engine
    engine.dispose()


@pytest.fixture(scope="function")
def db_session(db_engine: Engine):
    connection = db_engine.connect()
    # Start transaction and not commit it to rollback automatically
    transaction = connection.begin()
    session = sessionmaker(autocommit=False, autoflush=False, bind=connection)()

    yield session

    session.close()
    transaction.close()
    connection.close()


@pytest.fixture(scope="function")
def test_client(db_session: Session) -> TestClient:
    """Create a test http client"""
    app.dependency_overrides[get_db] = lambda: db_session
    return TestClient(app)


@pytest.fixture
def generator(db_session: Session) -> DataGenerator:
    return DataGenerator(db_session)
