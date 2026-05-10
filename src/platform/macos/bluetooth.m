#import <IOBluetooth/IOBluetooth.h>
#include <stdio.h>
#include <unistd.h>

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
    self.responseReceived = YES;
    const uint8_t *b = (const uint8_t *)dataPointer;
    fprintf(stderr, "[bt] received %zu byte(s):", dataLength);
    for (size_t i = 0; i < dataLength; i++) fprintf(stderr, " %02X", b[i]);
    fprintf(stderr, "\n");
}
@end

// ── shared SDP + channel-open helper ─────────────────────────────────────────

static IOBluetoothRFCOMMChannel *open_channel(const char *addr_cstr,
                                               OpenDelegate *delegate) {
    NSString *addr = [NSString stringWithUTF8String:addr_cstr];
    IOBluetoothDevice *dev = [IOBluetoothDevice deviceWithAddressString:addr];
    if (!dev) { fprintf(stderr, "[bt] device not found: %s\n", addr_cstr); return nil; }

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
        fprintf(stderr, "[bt] SPP channel from SDP: %u\n", (unsigned)channelID);
    } else {
        fprintf(stderr, "[bt] SDP lookup failed; falling back to channel %u\n", (unsigned)channelID);
    }

    IOBluetoothRFCOMMChannel *channel = nil;
    IOReturn r = [dev openRFCOMMChannelAsync:&channel withChannelID:channelID delegate:delegate];
    if (r != kIOReturnSuccess) {
        fprintf(stderr, "[bt] openRFCOMMChannelAsync failed: %d\n", r);
        return nil;
    }

    NSDate *deadline = [NSDate dateWithTimeIntervalSinceNow:10.0];
    while (!delegate.done && [deadline timeIntervalSinceNow] > 0)
        [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.05]];

    if (!delegate.connected) {
        fprintf(stderr, "[bt] channel open failed or timed out\n");
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

        fprintf(stderr, "[bt] RFCOMM channel open — sending %d packet(s)\n", count);
        for (int i = 0; i < count; i++) {
            IOReturn wr = [channel writeSync:(void *)packets[i] length:sizes[i]];
            if (wr != kIOReturnSuccess) {
                fprintf(stderr, "[bt] writeSync packet %d failed: %d\n", i, wr);
                [channel closeChannel];
                return -2;
            }
            fprintf(stderr, "[bt] sent packet %d/%d (%u bytes)\n", i + 1, count, (unsigned)sizes[i]);
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
            fprintf(stderr, "[bt] query writeSync failed: %d\n", wr);
            [channel closeChannel];
            return -2;
        }
        fprintf(stderr, "[bt] query sent %u byte(s), waiting for response...\n", (unsigned)cmd_len);

        NSDate *deadline = [NSDate dateWithTimeIntervalSinceNow:(double)timeout_ms / 1000.0];
        while (!delegate.responseReceived && [deadline timeIntervalSinceNow] > 0)
            [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.05]];

        if (!delegate.responseReceived) {
            fprintf(stderr, "[bt] query timed out — no response\n");
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
            fprintf(stderr, "[bt] no paired devices found\n");
            return;
        }
        for (IOBluetoothDevice *dev in devices) {
            printf("%s  %s\n",
                   [dev.addressString UTF8String] ?: "??:??:??:??:??:??",
                   [[dev name] UTF8String] ?: "<unnamed>");
        }
    }
}
