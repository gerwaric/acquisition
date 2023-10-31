/*
	Copyright 2014 Ilya Zhuravlev

	This file is part of Acquisition.

	Acquisition is free software: you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	Acquisition is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#pragma once

#ifndef VERSION_H
#define VERSION_H

// These control if this build will expire, and need to be defined before
// any other acquisition headers are included.
#define TRIAL_VERSION               1
#define TRIAL_EXPIRATION_DAYS       30

extern const int VERSION_CODE;
extern const char VERSION_NAME[];

extern const char UPDATE_CHECK_URL[];
extern const char UPDATE_DOWNLOAD_LOCATION[];

#endif
