package com.chunkzero.window.minestom

import com.chunkzero.window.host.testkit.HostConformance
import net.minestom.server.item.ItemStack

class MinestomHostConformanceTest : HostConformance<ItemStack>(::MinestomFixture)
