#!/usr/bin/python3

from os import path
import argparse
import sys
import struct
import hashlib

ROM_FILE_NAME = 'baserom'

VERSIONS_HASHES = {
    "NTSC 0.9 RC":      "21f7b4a4ff463464bfc23498c1ab9da1", # a.k.a. NTSC 1.0 RC
    "NTSC J 0.9 RC":    None,
    "NTSC 1.0":         "5bd1fe107bf8106b2ab6650abecd54d6", # Need to double check.
    "NTSC J 1.0":       None,
    "NTSC 1.1":         "721fdcc6f5f34be55c43a807f2a16af4", # Need to double check.
    "NTSC J 1.1":       None,
    "PAL 1.0":          "e040de91a74b61e3201db0e2323f768a",
    "NTSC 1.2":         "57a9719ad547c516342e1a15d5c28c3d",
    "NTSC J 1.2":       None,
    "PAL 1.1":          "d714580dd74c2c033f5e1b6dc0aeac77", # Need to double check.
    "PAL WII 1.1":      None,
    "JP GC":            "33fb7852c180b18ea0b9620b630f413f",
    "JP MQ":            "69895c5c78442260f6eafb2506dc482a",
    "USA GC":           "cd09029edcfb7c097ac01986a0f83d3f",
    "USA MQ":           "da35577fe54579f6a266931cc75f512d",
    "PAL GC DBG":       None,
    "PAL MQ DBG":       "f0b7f35375f9cc8ca1b2d59d78e35405",
    "PAL GC DBG2":      "3c10b67a76616ae2c162def7528724cf",
    "PAL GC":           "cd09029edcfb7c097ac01986a0f83d3f",
    "PAL MQ":           "1618403427e4344a57833043db5ce3c3", # I think it's right.
    "JP GC CE":         "0c13e0449a28ea5b925cdb8af8d29768", # Zelda collection
    "IQUE CN":          "0ab48b2d44a74b3bb2d384f6170c2742",
    "IQUE TW":          "a475e9f8615513666a265c464708ae8f",

    # MM
    "MM JP 1.0":        "15d1a2217cad61c39cfecbffa0703e25",
    "MM JP 1.1":        "c38a7f6f6b61862ea383a75cdf888279",
    #"MM USA KIOSK":    None,
    "MM USA":      "2a0a8acb61538235bc1094d297fb6556",
    #"MM PAL 1.0":       None,
    "MM PAL DBG":       None,
    #"MM PAL 1.1":       None,
    #"USA GC":           None,
    #"PAL GC":           None.
    #"JP GC":            None,
}

def getStrHash(byte_array):
    return str(hashlib.md5(byte_array).hexdigest())

def checkBaseromValid(edition):
    filename = f"{ROM_FILE_NAME}_{edition}.z64"
    version = edition.upper().replace("_", " ")

    if not path.exists(filename):
        filename = f"{ROM_FILE_NAME}.z64"
        if not path.exists(filename):
            return False

    with open(filename, mode="rb") as f:
        fileContent = bytearray(f.read())
        if getStrHash(fileContent) == VERSIONS_HASHES[version]:
            return True
    return False

def getOriginalRomFilename(edition):
    extensions = ["z64", "n64", "v64"]

    for ext in extensions:
        filename = f"{ROM_FILE_NAME}_{edition}_original.{ext}"
        if path.exists(filename):
            return filename

    for ext in extensions:
        filename = f"{ROM_FILE_NAME}_original.{ext}"
        if path.exists(filename):
            return filename

    return ""

def wordSwapRom(fileContent):
    words = str(int(len(fileContent)/4))
    little_byte_format = "<" + words + "I"
    big_byte_format = ">" + words + "I"
    tmp = struct.unpack_from(little_byte_format, fileContent, 0)
    struct.pack_into(big_byte_format, fileContent, 0, *tmp)
    return fileContent

def byteSwapRom(fileContent):
    halfwords = str(int(len(fileContent)/2))
    little_byte_format = "<" + halfwords + "H"
    big_byte_format = ">" + halfwords + "H"
    tmp = struct.unpack_from(little_byte_format, fileContent, 0)
    struct.pack_into(big_byte_format, fileContent, 0, *tmp)
    return fileContent

def perVersionFixes(fileContent, version):
    if version == "PAL MQ DBG":
        # Strip the overdump
        print("Stripping overdump...")
        fileContent = fileContent[0:0x3600000]

        # Patch the header
        print("Patching header...")
        fileContent[0x3E] = 0x50

    return fileContent


def fixBaserom(edition, check_hash):
    version = edition.upper().replace("_", " ")

    if check_hash and version not in VERSIONS_HASHES:
        print(f"Invalid version: {version}")
        sys.exit(1)

    romhash = VERSIONS_HASHES[version]
    if check_hash and romhash is None:
        print(f"Version {version} is not currently supported.")
        sys.exit(1)

    # If the baserom exists and is correct, we don't need to change anything
    if checkBaseromValid(edition):
        print("Found valid baserom - exiting early.")
        sys.exit(0)

    # Determine if we have a ROM file
    romFilename = getOriginalRomFilename(edition)
    if romFilename == "":
        print("Error: Could not find baserom original.")
        sys.exit(1)

    # Read in the original ROM
    print("File '" + romFilename + "' found.")
    with open(romFilename, mode="rb") as f:
        fileContent = bytearray(f.read())

    # Check if ROM needs to be byte/word swapped
    # Little-endian
    if fileContent[0] == 0x40:
        # Word Swap ROM
        print("ROM needs to be word swapped...")
        fileContent = wordSwapRom(fileContent)

        print("Word swapping done.")

    # Byte-swapped
    elif fileContent[0] == 0x37:
        # Byte Swap ROM
        print("ROM needs to be byte swapped...")
        fileContent = byteSwapRom(fileContent)

        print("Byte swapping done.")

    fileContent = perVersionFixes(fileContent, version)

    # Check to see if the ROM is unmodified.
    str_hash = getStrHash(bytearray(fileContent))
    if check_hash and str_hash != romhash:
        print(f"Error: Expected a hash of {romhash} but got {str_hash}. " +
            "The baserom has probably been tampered, find a new one.")
        sys.exit(1)

    # Write out our new ROM
    outRom = romFilename.replace("_original", "")[:-3] + "z64"
    print(f"Writing new ROM '{outRom}'.")
    with open(outRom, mode="wb") as file:
        file.write(bytes(fileContent))

    print("Done!")

def main():
    description = ""

    parser = argparse.ArgumentParser(description=description, formatter_class=argparse.RawTextHelpFormatter)
    choices = [x.lower().replace(" ", "_") for x in VERSIONS_HASHES]
    parser.add_argument("edition", help="Select the version of the game.", choices=choices, default="pal_mq_dbg", nargs='?')
    parser.add_argument("--dont-check-hash", help="Prevents the hash check. Useful if you want to convert a swapped rom.", action="store_true")
    args = parser.parse_args()

    fixBaserom(args.edition, not args.dont_check_hash)


if __name__ == "__main__":
    main()
