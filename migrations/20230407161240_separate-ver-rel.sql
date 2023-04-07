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
ALTER TABLE builds DROP pverl;
ALTER TABLE pkgs DROP verl;

ALTER TABLE pkgs ADD ver VARCHAR(255) NOT NULL DEFAULT '1';
ALTER TABLE pkgs ADD rel VARCHAR(255) NOT NULL DEFAULT '1';
ALTER TABLE builds ADD pver VARCHAR(255) NOT NULL DEFAULT '1';
ALTER TABLE builds ADD prel VARCHAR(255) NOT NULL DEFAULT '1';
