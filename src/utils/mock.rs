pub fn get_data() -> Vec<u8> {
            //a DEFINITELY valid journal entry at offset 0x1000
        //     let mut example_entry: Vec<u8> = vec![
        //         0xE0, // Year 2016
        //         0x01, // Month
        //         0x1F, // Day
        //         0x12, // Hour
        //         0x34, // Minute
        //         0x56, // Second
        //         0x01, // Event type (e.g., "Card Accepted")
        //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Reserved bytes
        //         // make them 2 for testing purposes
        //         0xE0, // Year 2016
        //         0x01, // Month
        //         0x1F, // Day
        //         0x12, // Hour
        //         0x34, // Minute
        //         0x56, // Second
        //         0x01, // Event type (e.g., "Card Accepted")
        //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Reserved bytes

        //     ];
        // let mut cards = vec![0xFF; 256];
        // let mut the_rest = vec![0xFF; 32510];
        
        // cards.append(&mut example_entry);
        // cards.append(&mut the_rest);

        // println!("Loading mock data: {:?}", &cards);
        // cards
        [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
        ].repeat(2048).to_vec()
        
}

