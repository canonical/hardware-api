#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License version 3, as
# published by the Free Software Foundation.
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
"""Script for importing test machine data using mocked C3 responses"""

import os
import json
import logging
from pathlib import Path
from typing import Any, Dict
from importlib.util import spec_from_file_location, module_from_spec
from requests_mock import Mocker

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


class C3TestDataImporter:
    """Handles importing test data with mocked C3 responses"""

    def __init__(self):
        self.script_dir = Path(__file__).parent
        self.test_data_dir = self.script_dir / "c3_test_data"
        self.c3_url = os.getenv("C3_URL", "https://certification.canonical.com")

    def load_json_file(self, filename: str) -> Dict[str, Any]:
        """Load and parse a JSON file from the test data directory"""
        filepath = self.test_data_dir / filename
        try:
            with open(filepath) as f:
                return json.load(f)
        except FileNotFoundError:
            logger.error(f"Test data file not found: {filepath}")
            raise
        except json.JSONDecodeError:
            logger.error(f"Invalid JSON in file: {filepath}")
            raise

    def setup_mock_endpoints(self, mocker: Mocker) -> None:
        """Configure mock endpoints for C3 API"""
        endpoints = {
            "/api/v2/public-certificates/": "public_certificates.json",
            "/api/v2/public-devices/?pagination=limitoffset&limit=1000": "public_devices.json",
            "/api/v2/cpuids/": "cpuids.json",
        }

        for endpoint, filename in endpoints.items():
            logger.info(f"Setting up mock endpoint: {endpoint}")
            try:
                mock_data = self.load_json_file(filename)
                mocker.get(f"{self.c3_url}{endpoint}", json=mock_data)
            except Exception as e:
                logger.error(f"Failed to setup mock for {endpoint}: {str(e)}")
                raise

    def import_data(self) -> None:
        """Run the import script with mocked responses"""
        import_script = self.script_dir / "import_from_c3.py"

        if not import_script.exists():
            raise FileNotFoundError(f"Import script not found: {import_script}")

        logger.info("Loading import script: %s", import_script)
        try:
            spec = spec_from_file_location("import_from_c3", import_script)
            if spec is None or spec.loader is None:
                raise ImportError("Failed to load import script specification")

            import_module = module_from_spec(spec)
            with Mocker() as mocker:
                self.setup_mock_endpoints(mocker)
                logger.info("Executing import script with mocked endpoints")
                spec.loader.exec_module(import_module)
                import_module.main()

        except Exception as e:
            logger.error("Error during import", exc_info=True)
            raise RuntimeError(f"Import failed: {str(e)}") from e

    def verify_test_data(self) -> None:
        """Verify that all required test data files exist and are valid JSON"""
        required_files = [
            "public_certificates.json",
            "public_devices.json",
            "cpuids.json",
        ]

        for filename in required_files:
            logger.info(f"Verifying test data file: {filename}")
            self.load_json_file(filename)


def main() -> None:
    """Main entry point"""
    logger.info("Starting C3 test data import")
    logger.info("C3_URL: %s", os.getenv("C3_URL", "not set"))
    logger.info("DB_URL: %s", os.getenv("DB_URL", "not set"))

    importer = C3TestDataImporter()

    try:
        # Verify test data before starting import
        importer.verify_test_data()
        # Run import with mocked responses
        importer.import_data()
        logger.info("Import completed successfully")

    except Exception as e:
        logger.error("Import failed: %s", str(e))
        raise SystemExit(1)


if __name__ == "__main__":
    main()
