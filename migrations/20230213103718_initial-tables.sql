-- This file is part of Madoguchi.
--
-- Madoguchi is free software: you can redistribute it and/or modify it under the terms of
-- the GNU General Public License as published by the Free Software Foundation, either
-- version 3 of the License, or (at your option) any later version.
--
-- Madoguchi is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
-- without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
-- See the GNU General Public License for more details.
--
-- You should have received a copy of the GNU General Public License along with Madoguchi.
-- If not, see <https://www.gnu.org/licenses/>.
--
CREATE TABLE repos (
	name	VARCHAR(255) PRIMARY KEY,
	link	VARCHAR(255) NOT NULL,
	gh		VARCHAR(255) NOT NULL
);

CREATE TABLE pkgs (
	name	VARCHAR(255) NOT NULL,
	repo	VARCHAR(255) REFERENCES repos(name),
	verl	VARCHAR(255) NOT NULL,
	arch	VARCHAR(225) NOT NULL,
	dirs	VARCHAR(255) NOT NULL,
	PRIMARY KEY (name, repo, verl, arch)
--	build	INT UNIQUE REFERENCES builds(id)
);

CREATE TABLE builds (
	id		SERIAL PRIMARY KEY,
	epoch	TIMESTAMP NOT NULL,
	pname	VARCHAR(255) NOT NULL,
	pverl	VARCHAR(255) NOT NULL,
	parch	VARCHAR(255) NOT NULL,
	repo	VARCHAR(255) REFERENCES repos(name),
	link	VARCHAR(255) NOT NULL,
	CONSTRAINT fk_pkg FOREIGN KEY (pname, repo, pverl, parch) REFERENCES pkgs (name, repo, verl, arch)

);

ALTER TABLE pkgs ADD build INT UNIQUE REFERENCES builds(id);
