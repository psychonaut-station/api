package byond

import (
	"bytes"
	"encoding/binary"
	"net"
	"time"
)

const TopicTypeNull = 0x0
const TopicTypeNumber = 0x2A
const TopicTypeString = 0x6

const BYOND_PACKET_HEADER_SIZE = 4

type ByondPacketHeader struct {
	PacketType uint16
	DataSize   uint16
}

func Topic(address string, data string) (error, int, []byte) {
	packetData := new(bytes.Buffer)

	packetData.WriteString("\x00\x83")
	packetData.WriteString("\x00" + string(len(data)+6))
	packetData.WriteString("\x00\x00\x00\x00\x00")
	packetData.WriteString(data)
	packetData.WriteString("\x00")

	conn, err := net.DialTimeout("tcp", address, 100*time.Millisecond)
	if conn == nil || err != nil {
		return err, TopicTypeNull, nil
	}

	conn.Write(packetData.Bytes())

	responseHeaderData := make([]byte, BYOND_PACKET_HEADER_SIZE)
	conn.Read(responseHeaderData)
	responseHeader := ByondPacketHeader{binary.BigEndian.Uint16(responseHeaderData[0:]), binary.BigEndian.Uint16(responseHeaderData[2:])}
	responseData := make([]byte, responseHeader.DataSize)
	conn.Read(responseData)

	responseDataType := TopicTypeNull
	if len(responseData) > 2 {
		responseDataType = int(responseData[0])
	}

	if responseDataType == TopicTypeNull {
		return nil, responseDataType, nil
	}

	return nil, responseDataType, responseData[1:]
}
