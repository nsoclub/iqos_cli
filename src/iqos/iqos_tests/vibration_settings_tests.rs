#[cfg(test)]
mod tests {
    use crate::iqos::iluma::VibrationSettings;
    use crate::iqos::error::IQOSError;

    #[test]
    fn test_parse_from_bytes_valid() {
        // テストケース1: すべての設定がオフのデータ
        let data1 = [0x00, 0x08, 0x84, 0x23, 0x10, 0x00, 0x00, 0x00, 0x00, 0x77];
        let settings = VibrationSettings::parse_from_bytes(&data1).expect("有効なデータを解析できませんでした");
        assert!(!settings.when_charging_start);
        assert!(!settings.when_heating_start);
        assert!(!settings.when_starting_to_use);
        assert!(!settings.when_puff_end);
        assert!(!settings.when_manually_terminated);

        // テストケース2: 加熱開始とパフエンドがオンのデータ
        let data2 = [0x00, 0x08, 0x84, 0x23, 0x10, 0x00, 0x00, 0x01, 0x01, 0x77];
        let settings = VibrationSettings::parse_from_bytes(&data2).expect("有効なデータを解析できませんでした");
        assert!(!settings.when_charging_start);
        assert!(settings.when_heating_start);
        assert!(!settings.when_starting_to_use);
        assert!(settings.when_puff_end);
        assert!(!settings.when_manually_terminated);

        // テストケース3: 使用開始と手動終了がオンのデータ
        let data3 = [0x00, 0x08, 0x84, 0x23, 0x10, 0x00, 0x00, 0x10, 0x10, 0x77];
        let settings = VibrationSettings::parse_from_bytes(&data3).expect("有効なデータを解析できませんでした");
        assert!(!settings.when_charging_start);
        assert!(!settings.when_heating_start);
        assert!(settings.when_starting_to_use);
        assert!(!settings.when_puff_end);
        assert!(settings.when_manually_terminated);

        // テストケース4: すべての設定がオンのデータ
        let data4 = [0x00, 0x08, 0x84, 0x23, 0x10, 0x00, 0x00, 0x11, 0x11, 0x77];
        let settings = VibrationSettings::parse_from_bytes(&data4).expect("有効なデータを解析できませんでした");
        assert!(!settings.when_charging_start); // charge設定は通知パケットに含まれていない
        assert!(settings.when_heating_start);
        assert!(settings.when_starting_to_use);
        assert!(settings.when_puff_end);
        assert!(settings.when_manually_terminated);
    }

    #[test]
    fn test_parse_from_bytes_invalid() {
        // テストケース1: データ長が短すぎる
        let data1 = [0x00, 0x08, 0x84, 0x23, 0x10];
        let result = VibrationSettings::parse_from_bytes(&data1);
        assert!(result.is_err());
        match result {
            Err(IQOSError::ConfigurationError(msg)) => {
                assert!(msg.contains("Data too short"));
            }
            _ => panic!("期待するエラー種別ではありません"),
        }

        // テストケース2: ヘッダーが不正
        let data2 = [0x01, 0x08, 0x84, 0x23, 0x10, 0x00, 0x00, 0x00, 0x00, 0x77];
        let result = VibrationSettings::parse_from_bytes(&data2);
        assert!(result.is_err());
        match result {
            Err(IQOSError::ConfigurationError(msg)) => {
                assert!(msg.contains("Invalid header"));
            }
            _ => panic!("期待するエラー種別ではありません"),
        }

        // テストケース3: 異なるコマンド型
        let data3 = [0x00, 0x08, 0x84, 0x24, 0x10, 0x00, 0x00, 0x00, 0x00, 0x77];
        let result = VibrationSettings::parse_from_bytes(&data3);
        assert!(result.is_err());
        match result {
            Err(IQOSError::ConfigurationError(msg)) => {
                assert!(msg.contains("Invalid header"));
            }
            _ => panic!("期待するエラー種別ではありません"),
        }
    }
}