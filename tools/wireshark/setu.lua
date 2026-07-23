local setu = Proto("setu", "Setu RPC")

-- Fields
local f_header = ProtoField.uint8("setu.flags", "Header", base.HEX)
local f_compressed  = ProtoField.bool("setu.flags.compressed", "Is Compressed", 8, nil, 0x01)
local f_trailer     = ProtoField.bool("setu.flags.trailer", "Is Trailer", 8, nil, 0x02)
local f_len_size    = ProtoField.uint8("setu.flags.len_size", "Length Size", base.DEC, nil, 0x0C)
local f_code        = ProtoField.uint8("setu.flags.code", "Trailer Code", base.DEC, nil, 0xF0)

local f_length      = ProtoField.uint32("setu.len", "Payload Length")
local f_payload     = ProtoField.bytes("setu.payload", "Payload Data")

setu.fields = { f_header, f_compressed, f_trailer, f_len_size, f_code, f_length, f_payload }

-- Dissector

--------------------------------------------------------

local Cursor = {}
Cursor.__index = Cursor

function Cursor.new(buffer)
    return setmetatable({
        buffer = buffer,
        offset = 0,
    }, Cursor)
end

function Cursor:read_u8()
    local value = self.buffer(self.offset, 1):uint()
    self.offset = self.offset + 1
    return value
end

function Cursor:read_bytes(len)
    local bytes = self.buffer(self.offset, len)
    self.offset = self.offset + len
    return bytes
end

function Cursor:remaining()
    return self.buffer:len() - self.offset
end

--------------------------------------------------------

local function parse_header(cursor, tree)
    local byte = cursor:read_u8()

    local header = tree:add( f_header, byte )

    local compressed = (byte & 0x01) ~= 0
    local trailer = (byte & 0x02) ~= 0
    local len_size = ((byte >> 2) & 0x03) + 1
    local code = (byte >> 4) & 0x0F

    header:add(f_compressed, compressed)
    header:add(f_trailer, trailer)
    header:add(f_len_size, len_size)
    header:add(f_code, code)

    return {
        compressed = compressed,
        trailer = trailer,
        len_size = len_size,
        code = code,
    }
end

local function parse_length(cursor, size)
    local length = 0

    for _ = 1, size do
        length = (length << 8) | cursor:read_u8()
    end

    return length
end

--------------------------------------------------------

function setu.dissector(buffer, pinfo, tree)
    pinfo.cols.protocol = setu.name
    
    -- Parse setu protocol here.
    local cursor = Cursor.new(buffer)

    local root = tree:add(setu, buffer(), "SETU Frame")

    while cursor:remaining() > 0 do
        local header = parse_header(cursor, root)
        local length = parse_length(cursor, header.len_size)
        
        root:add(f_length, length)
        root:add(f_payload, cursor:read_bytes(length))
    end
end

-- Register to HTTP/2 Media Types
local media_type_table = DissectorTable.get("media_type")
media_type_table:add("application/setu", setu)

print("[SETU] Lua dissector loaded")