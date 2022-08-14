#!/bin/bash
# Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
# 
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as
# published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
# 
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
# 
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

# publish.sh: grab bin from docker container, pack, sign and upload
# $2: binary version
# $3: Docker img tag
# $4: dummyserve username
# $5: dummyserve password

set -xEeuo  pipefail

DUMBSERVE_USERNAME=$4
DUMBSERVE_PASSWORD=$5
DUMBSERVE_HOST="https://$DUMBSERVE_USERNAME:$DUMBSERVE_PASSWORD@dl.batsense.net"

NAME=dummyserve

TMP_DIR=$(mktemp -d)
FILENAME="$NAMe-$2-linux-amd64"
TARBALL="$FILENAME.tar.gz"
TARGET_DIR="$TMP_DIR/$FILENAME"
DOCKER_IMG="realaravinth/$NAME:$3"


get_bin(){
	echo "[*] Grabbing binary"
	container_id=$(docker create $DOCKER_IMG)
	docker cp $container_id:/usr/local/bin/$NAME $TARGET_DIR/
	docker rm -v $container_id
}

copy() {
	echo "[*] Copying dist assets"
	cp README.md  $TARGET_DIR
	cp LICENSE.md $TARGET_DIR
	cp CHANGELOG.md $TARGET_DIR
	cp docker-compose.yml $TARGET_DIR

	mkdir $TARGET_DIR/docs
	cp docs/DEPLOYMENT.md $TARGET_DIR/docs
	cp docs/CONFIGURATION.md $TARGET_DIR/docs

	get_bin
}

pack() {
	echo "[*] Creating dist tarball"
	tar -cvzf $TARBALL $TARGET_DIR 
}

checksum() {
	echo "[*] Generating dist tarball checksum"
	sha256sum $TARBALL > $TARBALL.sha256
}

sign() {
	echo "[*] Signing dist tarball checksum"
	gpg --output $TARBALL.asc --sign --detach --armor $TARBALL
}

delete_dir() {
	curl --location --request DELETE "$DUMBSERVE_HOST/api/v1/files/delete" \
		--header 'Content-Type: application/json' \
		--data-raw "{
			\"path\": \"$1\"
		}"
}

upload_dist() {
	delete_dir $1

	for file in $TARBALL $TARBALL.asc $TARBALL.sha256
	do
		curl -v \
			-F upload=@$file  \
			"$DUMBSERVE_HOST/api/v1/files/upload?path=$1/"
	done
}




publish() {
	mkdir $TARGET_DIR
	copy
	pushd $TMP_DIR
	pack
	checksum
	sign
	publish
	popd
}

$1 $@
