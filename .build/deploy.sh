set -euo pipefail
IFS=$'\n\t'

function cleanup {
	    echo "🧹 Cleanup..."
	        rm -f ~/.gradle/gradle.properties eliorona-sign.asc
	}

trap cleanup SIGINT SIGTERM ERR EXIT

echo "🚀 Preparing to deploy..."

echo "🔑 Decrypting files..."

gpg --quiet --batch --yes --decrypt --passphrase="${GPG_SECRET}" \
	    --output eliorona-sign.asc .build/eliorona-sign.asc.gpg

mkdir ~/.gradle -p

gpg --quiet --batch --yes --decrypt --passphrase="${GPG_SECRET}" \
	    --output ~/.gradle/gradle.properties .build/usrgradle.properties.gpg

gpg --fast-import --no-tty --batch --yes eliorona-sign.asc

echo "📦 Publishing..."

./gradlew build
./gradlew generateChangelog github curseforge publishToModrinth
./gradlew uploadArchives -Psign

echo "✅ Done!"
