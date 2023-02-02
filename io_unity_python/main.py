try:
    import os
    import io_unity_python
    import fsb5
except ModuleNotFoundError as e:
    import sys
    sys.path.append("target/release")
    sys.path.append("python-fsb5")
    import io_unity_python
    import fsb5


def fsb5_extractor(input, output):

    fsb = fsb5.FSB5(input)

    print(fsb.header)

    # get the extension of samples based off the sound format specified in the header
    ext = fsb.get_sample_extension()

    # iterate over samples
    for sample in fsb.samples:
        # print sample properties
        print('''\t{sample.name}.{extension}:
        Frequency: {sample.frequency}
        Channels: {sample.channels}
        Samples: {sample.samples}'''.format(sample=sample, extension=ext))

        # rebuild the sample and save
        output_ext = ""
        if len(fsb.samples) > 1:
            output_ext += "_" + sample.name
        if not output.endswith("." + ext):
            output_ext += "." + ext
        with open(output + output_ext, 'wb') as f:
            rebuilt_sample = fsb.rebuild_sample(sample)
            f.write(rebuilt_sample)


out_path = ""
bundle_path = ""

uav = io_unity_python.UnityAssetViewer()

for root, dirs, files in os.walk(bundle_path):
    for name in files:
        if not name.lower().endswith('.bundle'):
            continue
        uav.add_bundle_file(os.path.join(root, name))

for objref in uav:
    if objref.get_class_id() == 83:
        obj = uav.deref_object_ref(objref)
        audio_name = uav.get_container_name_by_object_ref(objref)
        if audio_name == None and hasattr(obj, "m_Name"):
            audio_name = obj.m_Name
        if audio_name == None or len(audio_name) == 0:
            audio_name = "audio"
        print(audio_name)
        audio = io_unity_python.AudioClip(obj)
        path = os.path.join(out_path, audio_name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        fsb5_extractor(audio.get_audio_data(uav), path)
