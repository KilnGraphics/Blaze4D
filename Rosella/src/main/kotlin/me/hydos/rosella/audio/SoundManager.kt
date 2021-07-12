package me.hydos.rosella.audio

import me.hydos.rosella.render.resource.Resource
import org.apache.logging.log4j.LogManager
import org.lwjgl.BufferUtils
import org.lwjgl.openal.AL
import org.lwjgl.openal.AL10.*
import org.lwjgl.openal.ALC
import org.lwjgl.openal.ALC10.alcMakeContextCurrent
import org.lwjgl.openal.ALC11.alcCreateContext
import org.lwjgl.openal.ALC11.alcOpenDevice
import org.lwjgl.openal.EXTThreadLocalContext.alcSetThreadContext
import org.lwjgl.stb.STBVorbis.*
import org.lwjgl.stb.STBVorbisInfo
import org.lwjgl.system.MemoryUtil
import org.lwjgl.system.MemoryUtil.NULL
import java.nio.IntBuffer
import java.nio.ShortBuffer

object SoundManager {

    @JvmStatic
    fun initialize() {
        try {
            val device = alcOpenDevice(BufferUtils.createByteBuffer(1))
            check(device != NULL) { "Failed to open an OpenAL device." }

            val deviceCaps = ALC.createCapabilities(device)
            check(deviceCaps.OpenALC10)

            val context = alcCreateContext(device, null as IntBuffer?)
            val useTLC = deviceCaps.ALC_EXT_thread_local_context && alcSetThreadContext(context)

            if (!useTLC) {
                check(alcMakeContextCurrent(context))
            }

            AL.createCapabilities(deviceCaps, MemoryUtil::memCallocPointer)
        } catch (e: Exception) {
            LogManager.getFormatterLogger("Rosella").error("Unable to initialize sound manager: " + e.message)
        }
    }

    @JvmStatic
    fun playback(file: Resource) {
        val buffer: Int = alGenBuffers()
        val source: Int = alGenSources()
        STBVorbisInfo.malloc().use { info ->
            val pcm = readVorbis(file, info)

            alBufferData(
                buffer,
                if (info.channels() == 1) AL_FORMAT_MONO16 else AL_FORMAT_STEREO16,
                pcm,
                info.sample_rate()
            )
        }

        //set up source input
        alSourcei(source, AL_BUFFER, buffer)

        //play source
        alSourcePlay(source)

        Thread {
            while (true) {
                try {
                    Thread.sleep(1000)
                } catch (ignored: InterruptedException) {
                    break
                }

                if (alGetSourcei(source, AL_SOURCE_STATE) == AL_STOPPED) {
                    break
                }
            }

            alSourceStop(source)
            alDeleteSources(source)
            alDeleteBuffers(buffer)
        }.apply {
            isDaemon = true
            start()
        }
    }

    private fun readVorbis(resource: Resource, info: STBVorbisInfo): ShortBuffer {
        val vorbis = resource.readAllBytes(true)
        val error = BufferUtils.createIntBuffer(1)
        val decoder = stb_vorbis_open_memory(vorbis, error, null)

        if (decoder == NULL) {
            throw RuntimeException("Failed to open Ogg Vorbis file. Error: " + error[0])
        }

        stb_vorbis_get_info(decoder, info)

        val channels = info.channels()
        val pcm = BufferUtils.createShortBuffer(stb_vorbis_stream_length_in_samples(decoder) * channels)

        stb_vorbis_get_samples_short_interleaved(decoder, channels, pcm)
        stb_vorbis_close(decoder)

        return pcm
    }
}
