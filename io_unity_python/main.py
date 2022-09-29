try:
    import os
    import io_unity_python
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
        with open('{0}_{1}.{2}'.format(output, sample.name, ext), 'wb') as f:
            rebuilt_sample = fsb.rebuild_sample(sample)
            f.write(rebuilt_sample)


def get_a(obj,fs):
    banks = obj.get_audio_data(fs)
    name = obj.get_name()

    fsb5_extractor(banks, "/tmp/audio/"+name)

def handle(file_path):

    fs = io_unity_python.UnityFS.readfs(file_path)
    cab = fs.get_cab()

    for i in range(cab.get_object_count()):
        try:
            get_a(cab.get_raw_object_by_index(i),fs)
        except Exception as e:
            print(i)
            pass

for root, dirs, files in os.walk("/tmp/split_UnityDataAssetPack/assets/aa/Android/"):
    for name in files:
        if not name.lower().endswith('.bundle'):
            continue 
        handle(os.path.join(root, name))