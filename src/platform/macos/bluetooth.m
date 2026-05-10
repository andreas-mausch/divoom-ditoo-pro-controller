#import <IOBluetooth/IOBluetooth.h>
#include <stdio.h>
#include <unistd.h>

extern void bt_log(uint8_t level, const char *msg);
#define BT_DEBUG(fmt, ...) do { char _buf[512]; snprintf(_buf, sizeof(_buf), fmt, ##__VA_ARGS__); bt_log(0, _buf); } while(0)
#define BT_INFO(fmt,  ...) do { char _buf[512]; snprintf(_buf, sizeof(_buf), fmt, ##__VA_ARGS__); bt_log(1, _buf); } while(0)
#define BT_WARN(fmt,  ...) do { char _buf[512]; snprintf(_buf, sizeof(_buf), fmt, ##__VA_ARGS__); bt_log(2, _buf); } while(0)

@interface OpenDelegate : NSObject <IOBluetoothRFCOMMChannelDelegate>
@property (nonatomic, assign) BOOL done;
@property (nonatomic, assign) BOOL connected;
@property (nonatomic, strong) NSMutableData *responseBuffer;
@property (nonatomic, assign) BOOL responseReceived;
@end

@implementation OpenDelegate
- (instancetype)init {
    self = [super init];
    _responseBuffer = [NSMutableData data];
    return self;
}
- (void)rfcommChannelOpenComplete:(IOBluetoothRFCOMMChannel *)ch status:(IOReturn)error {
    self.connected = (error == kIOReturnSuccess);
    self.done = YES;
}
- (void)rfcommChannelClosed:(IOBluetoothRFCOMMChannel *)ch {
    self.done = YES;
}
- (void)rfcommChannelData:(IOBluetoothRFCOMMChannel *)ch
                     data:(void *)dataPointer
                   length:(size_t)dataLength {
    [self.responseBuffer appendBytes:dataPointer length:dataLength];
    const uint8_t *b = (const uint8_t *)self.responseBuffer.bytes;
    NSUInteger len = self.responseBuffer.length;
    if (len >= 4 && b[0] == 0x01 && b[len - 1] == 0x02) {
        self.responseReceived = YES;
    }
    BT_DEBUG("received chunk %zu byte(s), buffer now %lu byte(s)", dataLength, (unsigned long)len);
}
@end

// ── shared SDP + channel-open helper ─────────────────────────────────────────

static IOBluetoothRFCOMMChannel *open_channel(const char *addr_cstr,
                                               OpenDelegate *delegate) {
    NSString *addr = [NSString stringWithUTF8String:addr_cstr];
    IOBluetoothDevice *dev = [IOBluetoothDevice deviceWithAddressString:addr];
    if (!dev) { BT_WARN("device not found: %s", addr_cstr); return nil; }

    [dev performSDPQuery:nil];
    NSDate *sdpDeadline = [NSDate dateWithTimeIntervalSinceNow:8.0];
    while ([sdpDeadline timeIntervalSinceNow] > 0) {
        [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.1]];
        if ([dev getServiceRecordForUUID:[IOBluetoothSDPUUID uuid16:0x1101]]) break;
    }

    IOBluetoothSDPServiceRecord *sppRecord =
        [dev getServiceRecordForUUID:[IOBluetoothSDPUUID uuid16:0x1101]];
    BluetoothRFCOMMChannelID channelID = 2;
    if (sppRecord) {
        [sppRecord getRFCOMMChannelID:&channelID];
        BT_DEBUG("SPP channel from SDP: %u", (unsigned)channelID);
    } else {
        BT_WARN("SDP lookup failed; falling back to channel %u", (unsigned)channelID);
    }

    IOBluetoothRFCOMMChannel *channel = nil;
    IOReturn r = [dev openRFCOMMChannelAsync:&channel withChannelID:channelID delegate:delegate];
    if (r != kIOReturnSuccess) {
        BT_WARN("openRFCOMMChannelAsync failed: %d", r);
        return nil;
    }

    NSDate *deadline = [NSDate dateWithTimeIntervalSinceNow:10.0];
    while (!delegate.done && [deadline timeIntervalSinceNow] > 0)
        [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.05]];

    if (!delegate.connected) {
        BT_WARN("channel open failed or timed out");
        return nil;
    }
    return channel;
}

// ── bt_rfcomm_send ────────────────────────────────────────────────────────────

int bt_rfcomm_send(const char *addr_cstr,
                   const uint8_t * const *packets,
                   const uint16_t *sizes,
                   int count,
                   unsigned int delay_ms) {
    @autoreleasepool {
        OpenDelegate *delegate = [[OpenDelegate alloc] init];
        IOBluetoothRFCOMMChannel *channel = open_channel(addr_cstr, delegate);
        if (!channel) return -1;

        BT_INFO("RFCOMM channel open — sending %d packet(s)", count);
        for (int i = 0; i < count; i++) {
            IOReturn wr = [channel writeSync:(void *)packets[i] length:sizes[i]];
            if (wr != kIOReturnSuccess) {
                BT_WARN("writeSync packet %d failed: %d", i, wr);
                [channel closeChannel];
                return -2;
            }
            BT_DEBUG("sent packet %d/%d (%u bytes)", i + 1, count, (unsigned)sizes[i]);
            if (i < count - 1 && delay_ms > 0) usleep(delay_ms * 1000);
        }

        [channel closeChannel];
        return 0;
    }
}

// ── bt_rfcomm_query ───────────────────────────────────────────────────────────

int bt_rfcomm_query(const char *addr_cstr,
                    const uint8_t *cmd, uint16_t cmd_len,
                    uint8_t *out_buf, size_t out_buf_size, size_t *out_len,
                    unsigned int timeout_ms) {
    @autoreleasepool {
        *out_len = 0;
        OpenDelegate *delegate = [[OpenDelegate alloc] init];
        IOBluetoothRFCOMMChannel *channel = open_channel(addr_cstr, delegate);
        if (!channel) return -1;

        IOReturn wr = [channel writeSync:(void *)cmd length:cmd_len];
        if (wr != kIOReturnSuccess) {
            BT_WARN("query writeSync failed: %d", wr);
            [channel closeChannel];
            return -2;
        }
        BT_DEBUG("query sent %u byte(s), waiting for response...", (unsigned)cmd_len);

        NSDate *deadline = [NSDate dateWithTimeIntervalSinceNow:(double)timeout_ms / 1000.0];
        while (!delegate.responseReceived && [deadline timeIntervalSinceNow] > 0)
            [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.05]];

        if (!delegate.responseReceived) {
            BT_WARN("query timed out — no response");
            [channel closeChannel];
            return -3;
        }

        NSUInteger available = delegate.responseBuffer.length;
        size_t copy_len = available < out_buf_size ? available : out_buf_size;
        memcpy(out_buf, delegate.responseBuffer.bytes, copy_len);
        *out_len = copy_len;

        [channel closeChannel];
        return 0;
    }
}

// ── bt_list_devices ───────────────────────────────────────────────────────────

void bt_list_devices(void) {
    @autoreleasepool {
        NSArray *devices = [IOBluetoothDevice pairedDevices];
        if (!devices || devices.count == 0) {
            BT_INFO("no paired devices found");
            return;
        }
        for (IOBluetoothDevice *dev in devices) {
            printf("%s  %s\n",
                   [dev.addressString UTF8String] ?: "??:??:??:??:??:??",
                   [[dev name] UTF8String] ?: "<unnamed>");
        }
    }
}
